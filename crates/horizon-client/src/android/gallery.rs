//! Publish F2 PNGs through MediaStore, so they show up in the phone's gallery without asking for
//! broad storage access on Android 10 and up. The screenshot stays in the game folder as well;
//! this only copies it to `Pictures/HorizonAndroid` (or `Pictures` on Android 9 and older).
//!
//! Ported from the Volt Launcher tree (see the harvest notes), reworked to use this game's
//! `with_activity` hook and translation keys instead of hardcoded English.
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JObject, JValue},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

/// A screenshot that has been published (file name), could not be (reason), or waited on storage
/// permission for too long.
pub enum Outcome {
    Saved(String),
    Failed(String),
    PermissionDenied,
}

/// Why asking Android to publish a screenshot failed before a thread took over.
pub enum Failure {
    /// The activity was not reachable (the game is not running on a NativeActivity).
    Unavailable,
    /// The publishing thread could not start.
    Spawn(String),
}

static RESULTS: OnceLock<Mutex<Vec<Outcome>>> = OnceLock::new();
/// A storage permission dialog is up; do not open another one for the next screenshot.
static PERMISSION_PENDING: AtomicBool = AtomicBool::new(false);

struct Request {
    vm: JavaVM,
    activity: GlobalRef,
    resolver: GlobalRef,
    sdk: i32,
    legacy_dir: Option<PathBuf>,
    file: PathBuf,
}

enum WriteError {
    /// The player did not answer the storage permission dialog in time.
    Permission,
    Message(String),
}

impl From<jni::errors::Error> for WriteError {
    fn from(error: jni::errors::Error) -> Self {
        Self::Message(error.to_string())
    }
}

impl From<std::io::Error> for WriteError {
    fn from(error: std::io::Error) -> Self {
        Self::Message(error.to_string())
    }
}

/// Whether the app may write shared storage (Android 9 and older ask for it at runtime).
fn permission(env: &mut JNIEnv<'_>, activity: &JObject<'_>) -> jni::errors::Result<bool> {
    let name = JObject::from(env.new_string("android.permission.WRITE_EXTERNAL_STORAGE")?);
    Ok(env
        .call_method(
            activity,
            "checkSelfPermission",
            "(Ljava/lang/String;)I",
            &[JValue::Object(&name)],
        )?
        .i()?
        == 0)
}

/// Copy `file` into the system gallery. Returns once the background copy has been started; its
/// result arrives from [`poll`] on a later frame.
pub fn save(file: &Path) -> std::result::Result<(), Failure> {
    let request = super::with_activity(|env, activity| {
        // A screenshot can be taken many times; free this call's local references afterwards.
        env.with_local_frame(32, |env| prepare(env, activity, file))
    });
    let Some(request) = request else {
        PERMISSION_PENDING.store(false, Ordering::Relaxed);
        return Err(Failure::Unavailable);
    };
    std::thread::Builder::new()
        .name("Screenshot gallery".into())
        .spawn(move || {
            let outcome = match write(request) {
                Ok(name) => Outcome::Saved(name),
                Err(WriteError::Permission) => Outcome::PermissionDenied,
                Err(WriteError::Message(message)) => Outcome::Failed(message),
            };
            PERMISSION_PENDING.store(false, Ordering::Relaxed);
            RESULTS
                .get_or_init(Mutex::default)
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(outcome);
        })
        .map_err(|error| {
            PERMISSION_PENDING.store(false, Ordering::Relaxed);
            Failure::Spawn(error.to_string())
        })?;
    Ok(())
}

/// The results of copies finished since the last call.
pub fn poll() -> Vec<Outcome> {
    std::mem::take(
        &mut *RESULTS
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner()),
    )
}

/// What the background thread needs from the activity, made on the calling thread while the
/// activity is guaranteed to be alive.
fn prepare(
    env: &mut JNIEnv<'_>,
    activity: &JObject<'_>,
    file: &Path,
) -> jni::errors::Result<Request> {
    let sdk = env
        .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?
        .i()?;
    if sdk <= 28
        && !permission(env, activity)?
        && !PERMISSION_PENDING.swap(true, Ordering::Relaxed)
    {
        let permissions = env.new_object_array(1, "java/lang/String", JObject::null())?;
        env.set_object_array_element(
            &permissions,
            0,
            env.new_string("android.permission.WRITE_EXTERNAL_STORAGE")?,
        )?;
        env.call_method(
            activity,
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[JValue::Object(&permissions), JValue::Int(202)],
        )?;
    }
    let resolver = env
        .call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )?
        .l()?;
    let legacy_dir = if sdk <= 28 {
        let pictures = JObject::from(env.new_string("Pictures")?);
        let dir = env
            .call_static_method(
                "android/os/Environment",
                "getExternalStoragePublicDirectory",
                "(Ljava/lang/String;)Ljava/io/File;",
                &[JValue::Object(&pictures)],
            )?
            .l()?;
        let path = env
            .call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        let value = String::from(env.get_string(&path.into())?);
        Some(PathBuf::from(value).join("HorizonAndroid"))
    } else {
        None
    };
    Ok(Request {
        vm: env.get_java_vm()?,
        activity: env.new_global_ref(activity)?,
        resolver: env.new_global_ref(resolver)?,
        sdk,
        legacy_dir,
        file: file.to_owned(),
    })
}

fn put(
    env: &mut JNIEnv<'_>,
    values: &JObject<'_>,
    key: &str,
    text: &str,
) -> jni::errors::Result<()> {
    let key = JObject::from(env.new_string(key)?);
    let text = JObject::from(env.new_string(text)?);
    env.call_method(
        values,
        "put",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[JValue::Object(&key), JValue::Object(&text)],
    )?;
    Ok(())
}

fn pending(env: &mut JNIEnv<'_>, values: &JObject<'_>, value: i32) -> jni::errors::Result<()> {
    let key = JObject::from(env.new_string("is_pending")?);
    let number = env
        .call_static_method(
            "java/lang/Integer",
            "valueOf",
            "(I)Ljava/lang/Integer;",
            &[JValue::Int(value)],
        )?
        .l()?;
    env.call_method(
        values,
        "put",
        "(Ljava/lang/String;Ljava/lang/Integer;)V",
        &[JValue::Object(&key), JValue::Object(&number)],
    )?;
    Ok(())
}

fn write(request: Request) -> std::result::Result<String, WriteError> {
    use std::os::fd::FromRawFd;
    let mut guard = request
        .vm
        .attach_current_thread()
        .map_err(|error| WriteError::Message(error.to_string()))?;
    let env = &mut *guard;
    if request.sdk <= 28 {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !permission(env, request.activity.as_obj())? {
            if std::time::Instant::now() >= deadline {
                return Err(WriteError::Permission);
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
    let mut input = std::fs::File::open(&request.file)?;
    let name = request
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| WriteError::Message("Invalid screenshot name".into()))?;
    let mut row = JObject::null();
    let mut reserved_legacy: Option<PathBuf> = None;
    let result = (|| -> std::result::Result<String, WriteError> {
        let values = env.new_object("android/content/ContentValues", "()V", &[])?;
        put(env, &values, "_display_name", name)?;
        put(env, &values, "mime_type", "image/png")?;
        if request.sdk >= 29 {
            put(env, &values, "relative_path", "Pictures/HorizonAndroid")?;
            pending(env, &values, 1)?;
        } else if let Some(dir) = &request.legacy_dir {
            std::fs::create_dir_all(dir)?;
            let mut output = dir.join(name);
            for n in 1.. {
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&output)
                {
                    Ok(_) => {
                        reserved_legacy = Some(output.clone());
                        break;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e.into()),
                }
                output = dir.join(format!(
                    "{}_{n}.{}",
                    request
                        .file
                        .file_stem()
                        .map_or(String::new(), |s| s.to_string_lossy().into_owned()),
                    request
                        .file
                        .extension()
                        .map_or(String::new(), |s| s.to_string_lossy().into_owned())
                ));
            }
            put(env, &values, "_data", &output.to_string_lossy())?;
        }
        let images = env
            .get_static_field(
                "android/provider/MediaStore$Images$Media",
                "EXTERNAL_CONTENT_URI",
                "Landroid/net/Uri;",
            )?
            .l()?;
        row = env
            .call_method(
                request.resolver.as_obj(),
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[JValue::Object(&images), JValue::Object(&values)],
            )?
            .l()?;
        if row.is_null() {
            return Err(WriteError::Message(
                "MediaStore did not create an image".into(),
            ));
        }
        let mode = JObject::from(env.new_string("w")?);
        let fd = env
            .call_method(
                request.resolver.as_obj(),
                "openFileDescriptor",
                "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
                &[JValue::Object(&row), JValue::Object(&mode)],
            )?
            .l()?;
        let raw = env.call_method(&fd, "detachFd", "()I", &[])?.i()?;
        if raw < 0 {
            return Err(WriteError::Message(
                "MediaStore did not provide a writable file".into(),
            ));
        }
        // detachFd transfers ownership to this File, including on copy failure.
        // SAFETY: `raw` is a fresh descriptor that the ParcelFileDescriptor gave up.
        let mut output = unsafe { std::fs::File::from_raw_fd(raw) };
        std::io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        drop(output);
        if request.sdk >= 29 {
            let published = env.new_object("android/content/ContentValues", "()V", &[])?;
            pending(env, &published, 0)?;
            env.call_method(
                request.resolver.as_obj(),
                "update",
                "(Landroid/net/Uri;Landroid/content/ContentValues;Ljava/lang/String;[Ljava/lang/String;)I",
                &[
                    JValue::Object(&row),
                    JValue::Object(&published),
                    JValue::Object(&JObject::null()),
                    JValue::Object(&JObject::null()),
                ],
            )?;
        }
        Ok(name.to_owned())
    })();
    if result.is_err() && !row.is_null() {
        let _ = env.exception_clear();
        let _ = env.call_method(
            request.resolver.as_obj(),
            "delete",
            "(Landroid/net/Uri;Ljava/lang/String;[Ljava/lang/String;)I",
            &[
                JValue::Object(&row),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null()),
            ],
        );
        let _ = env.exception_clear();
    }
    if result.is_err()
        && let Some(path) = reserved_legacy
    {
        let _ = std::fs::remove_file(path);
    }
    result
}

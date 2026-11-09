//! Bounded local diagnostics in the game folder's `performance/`: a redacted copy of the log,
//! the session's state, and Android's exit history (exit_reports.rs) beside it, so a bug report
//! can say what the previous run was doing when the phone ended it.
//!
//! No signal handlers: Android owns native tombstones and low-memory exits. Nothing leaves the
//! device on its own; the files are for a player to attach to a report.
//!
//! Ported from the Volt Launcher tree (see the harvest notes), trimmed to what this game reads
//! itself (its launcher-side export and status pages are not part of this port).
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const LOG_LIMIT: u64 = 512 * 1024;
static DROPPED_LOGS: AtomicU64 = AtomicU64::new(0);
static STATE: Mutex<Option<State>> = Mutex::new(None);
/// Lines logged before `initialize`, kept so the copy starts at the process's start.
static EARLY: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

struct State {
    sender: mpsc::SyncSender<Event>,
    session: Value,
}
enum Event {
    Log(String),
    Session(Value),
    Flush(mpsc::SyncSender<()>),
}

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// `ApplicationExitInfo.getReason`'s numbers (ApplicationExitInfo).
pub fn reason_name(reason: i32) -> &'static str {
    match reason {
        1 => "exit_self",
        2 => "signaled",
        3 => "low_memory",
        4 => "java_crash",
        5 => "native_crash",
        6 => "anr",
        7 => "initialization_failure",
        8 => "permission_change",
        9 => "excessive_resource_usage",
        10 => "user_requested",
        11 => "user_stopped",
        12 => "dependency_died",
        13 => "other",
        14 => "freezer",
        15 => "package_state_change",
        16 => "package_updated",
        _ => "unknown",
    }
}

/// The last `limit` bytes of a file, read from there (a 512 KB tail, not the whole log).
pub fn tail(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    file.seek(SeekFrom::Start(size.saturating_sub(limit)))?;
    let mut out = Vec::new();
    file.take(limit).read_to_end(&mut out)?;
    Ok(out)
}

fn atomic_json(path: &Path, value: &Value) -> io::Result<()> {
    let temp = path.with_extension("partial");
    fs::write(&temp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(temp, path)
}

/// Starts the copy in `home/performance`, keeping the previous run's files aside. Called once,
/// before the log's first line lands in the worker.
pub fn initialize(home: &Path, device: Value) -> io::Result<()> {
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if state.is_some() {
        return Ok(());
    }
    let dir = home.join("performance");
    fs::create_dir_all(&dir)?;
    for (old, new) in [
        ("current-session.json", "previous-session.json"),
        ("diagnostic.log", "previous-diagnostic.log"),
    ] {
        if let Ok(data) = tail(&dir.join(old), LOG_LIMIT) {
            fs::write(dir.join(new), data)?;
        }
    }
    let session = json!({
        "schema": 1,
        "pid": std::process::id(),
        "start_ms": timestamp(),
        "version": env!("CARGO_PKG_VERSION"),
        "abi": std::env::consts::ARCH,
        "clean_exit": false,
        "device": device,
    });
    atomic_json(&dir.join("current-session.json"), &session)?;
    let (sender, receiver) = mpsc::sync_channel(128);
    let writer_dir = dir.clone();
    std::thread::Builder::new()
        .name("Local diagnostics".into())
        .spawn(move || {
            let mut log = File::create(writer_dir.join("diagnostic.log")).ok();
            let mut size = 0u64;
            for event in receiver {
                match event {
                    Event::Log(line) => {
                        let line = redact(&line);
                        if size + line.len() as u64 + 1 > LOG_LIMIT {
                            drop(log.take());
                            let _ = fs::rename(
                                writer_dir.join("diagnostic.log"),
                                writer_dir.join("previous-diagnostic.log"),
                            );
                            log = File::create(writer_dir.join("diagnostic.log")).ok();
                            size = 0;
                        }
                        if let Some(file) = log.as_mut() {
                            let _ = writeln!(file, "{line}");
                        }
                        size += line.len() as u64 + 1;
                    }
                    Event::Session(mut value) => {
                        value["memory"] = memory();
                        let _ = atomic_json(&writer_dir.join("current-session.json"), &value);
                    }
                    Event::Flush(done) => {
                        if let Some(file) = log.as_mut() {
                            let _ = file.flush();
                        }
                        let _ = done.send(());
                    }
                }
            }
        })?;
    for line in EARLY.lock().unwrap_or_else(|e| e.into_inner()).drain(..) {
        let _ = sender.try_send(Event::Log(line));
    }
    *state = Some(State { sender, session });
    Ok(())
}

/// Called by the logger; never waits for the state lock or a full queue: a busy diagnostic
/// state drops the line rather than making rendering wait.
pub fn log_line(level: log::Level, target: &str, message: &str) {
    if level > log::Level::Info {
        return;
    }
    let line = format!(
        "{} [{level}] {target}: {}",
        timestamp(),
        message.chars().take(4096).collect::<String>()
    );
    let guard = match STATE.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::WouldBlock) => {
            DROPPED_LOGS.fetch_add(1, Ordering::Relaxed);
            return;
        }
        Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
    };
    if let Some(state) = guard.as_ref() {
        if state.sender.try_send(Event::Log(line)).is_err() {
            DROPPED_LOGS.fetch_add(1, Ordering::Relaxed);
        }
        return;
    }
    drop(guard);
    let mut early = EARLY.lock().unwrap_or_else(|e| e.into_inner());
    if early.len() == 32 {
        early.pop_front();
    }
    early.push_back(line);
}

fn memory() -> Value {
    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut values = serde_json::Map::new();
    for line in status.lines() {
        if let Some((key, value)) = line.split_once(':')
            && matches!(key, "VmRSS" | "VmHWM" | "VmSize" | "VmPeak" | "Threads")
        {
            values.insert(key.into(), value.trim().into());
        }
    }
    json!(values)
}

/// The game is ending: mark the session clean, so the next start can tell a crash from a quit,
/// and wait (half a second at most) for the copy to reach the disk.
pub fn finish() {
    if let Some(state) = STATE.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        state.session["clean_exit"] = true.into();
        state.session["dropped_log_lines"] = DROPPED_LOGS.load(Ordering::Relaxed).into();
        state.session["updated_ms"] = timestamp().into();
        let _ = state.sender.try_send(Event::Session(state.session.clone()));
    }
    flush();
}

fn flush() {
    let (tx, rx) = mpsc::sync_channel(1);
    if let Some(state) = STATE.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        let _ = state.sender.try_send(Event::Flush(tx));
    }
    let _ = rx.recv_timeout(Duration::from_millis(500));
}

/// Whether the previous run ended without marking itself clean: a crash, a frozen or killed
/// process, or a battery pull. The next start says so in the log.
pub fn previous_unclean(home: &Path) -> bool {
    fs::read(home.join("performance/previous-session.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .is_some_and(|v| v["clean_exit"] != true)
}

/// Best-effort redaction of a diagnostic line: credentials, server addresses and this device's
/// private paths go, file names and native frame names stay. Not a promise that arbitrary
/// third-party text is public; the files say to review before sharing.
pub fn redact(text: &str) -> String {
    static PRIVATE: OnceLock<regex::Regex> = OnceLock::new();
    static ADDRESS: OnceLock<regex::Regex> = OnceLock::new();
    static DOMAIN: OnceLock<regex::Regex> = OnceLock::new();
    let private = PRIVATE.get_or_init(|| {
        regex::Regex::new(
            r"(?i)access.?token|refresh.?token|authorization|bearer |password|client.?secret|xbl3\.0|chat message|connecting to|joining server|player name|username",
        )
        .unwrap()
    });
    let address = ADDRESS.get_or_init(|| {
        regex::Regex::new(
            r#"(?i)https?://[^\s"<>]+|\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?::[0-9]+)?|/data/(?:user|user_de)/[^\s"<>]+|/storage/emulated/[^\s"<>]+|[a-z]:\\Users\\[^\\\s]+"#,
        )
        .unwrap()
    });
    let domain = DOMAIN.get_or_init(|| {
        regex::Regex::new(r"(?i)\b(?:[a-z0-9_-]+\.)+[a-z]{2,}\b|\[[a-f0-9:]+\](?::[0-9]+)?")
            .unwrap()
    });
    text.lines()
        .map(|line| {
            if private.is_match(line) {
                "[private log line removed]".into()
            } else {
                let line = address.replace_all(line, "[redacted]");
                domain
                    .replace_all(&line, |caps: &regex::Captures<'_>| {
                        let name = &caps[0];
                        let extension = name.rsplit('.').next().unwrap_or_default();
                        if matches!(
                            extension,
                            "rs" | "so" | "dll" | "wgsl" | "json" | "jsonl" | "txt" | "java"
                                | "kt" | "toml" | "trace" | "log" | "zip" | "apk"
                        ) {
                            name.to_owned()
                        } else {
                            "[address removed]".into()
                        }
                    })
                    .into_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redaction_removes_credentials_and_addresses_but_keeps_native_frames() {
        let text = redact(
            "access_token=sealed-token-value\nconnection 192.0.2.7:25565 leo.spcfy.eu https://example.org/private\n/data/user/0/android.horizon.ramil/files/saves/PrivateWorld\nC:\\Users\\Alice\\source\\game.rs\n#00 pc 00004abc libvulkan.so (BuildId: abcdef123456)",
        );
        for secret in [
            "sealed-token-value",
            "192.0.2.7",
            "leo.spcfy.eu",
            "example.org",
            "PrivateWorld",
            "Alice",
        ] {
            assert!(!text.contains(secret), "{secret}: {text}");
        }
        assert!(text.contains("libvulkan.so"));
        assert!(text.contains("abcdef123456"));
    }

    #[test]
    fn exit_reasons_distinguish_native_java_and_low_memory() {
        assert_eq!(reason_name(3), "low_memory");
        assert_eq!(reason_name(4), "java_crash");
        assert_eq!(reason_name(5), "native_crash");
        assert_eq!(reason_name(6), "anr");
        assert_eq!(reason_name(999), "unknown");
    }
}

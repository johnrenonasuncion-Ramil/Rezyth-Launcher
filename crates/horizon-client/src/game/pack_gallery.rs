//! The Modrinth resource-pack gallery's workers and effects (the page, at
//! `Options > Resource Packs > Download Packs`): search and version requests,
//! a download per install with its SHA-512 checked, the sidecar of what was
//! installed (`pack-gallery.json`), the update check and the auto-enable. The
//! state the page draws is `horizon_client::pack_gallery::Gallery`, in the
//! `OptionsUi` the event came from (the game's or the startup menu's).
use super::{Game, packs};
use horizon_client::options_ui::Event;
use horizon_client::pack_gallery as gallery;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

/// How long after typing a search runs.
const DEBOUNCE: Duration = Duration::from_millis(450);
/// How long an update check's answer is trusted before it is asked again.
const UPDATE_EVERY: Duration = Duration::from_secs(60);
/// Who Modrinth sees: it asks for the app and where to reach its makers.
const USER_AGENT: &str = "Rezyth-Launcher (github.com/johnrenonasuncion-Ramil/Rezyth-Launcher)";
/// The sidecar of what the gallery installed, in the game folder.
const INDEX: &str = "pack-gallery.json";

/// What a worker sends back; [`Game::poll_pack_gallery`] applies it.
pub(super) enum Msg {
    Search {
        menu: bool,
        generation: u64,
        offset: usize,
        query: String,
        result: Result<(Vec<gallery::Pack>, u64), String>,
    },
    Progress {
        menu: bool,
        project_id: String,
        done: u64,
        total: u64,
    },
    Installed {
        menu: bool,
        record: gallery::Installed,
        replace: Option<String>,
    },
    InstallFailed {
        menu: bool,
        project_id: String,
        error: String,
    },
    Updates {
        menu: bool,
        updates: Vec<gallery::Update>,
    },
}

/// The gallery's worker half on `Game`: the channel, the running search's
/// number per options UI, and when the installed packs were last checked.
pub(super) struct PackGallery {
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    generations: [u64; 2],
    checked: Option<Instant>,
}

impl PackGallery {
    pub(super) fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            tx,
            rx,
            generations: [0, 0],
            checked: None,
        }
    }
}

impl Default for PackGallery {
    fn default() -> Self {
        Self::new()
    }
}

fn slot(menu: bool) -> usize {
    usize::from(menu)
}

impl Game<'_> {
    /// What the gallery's events ask of the game; other events go on.
    pub(super) fn pack_gallery_event(&mut self, menu: bool, event: &Event) -> bool {
        match event {
            Event::ListPackGallery => self.list_pack_gallery(menu),
            Event::PackGalleryInstall(i) => self.install_gallery_result(menu, *i),
            Event::PackGalleryUpdate(file) => self.install_gallery_update(menu, file),
            Event::PackGalleryMore => self.start_gallery_search(menu, true),
            Event::PackGalleryRetry => self.start_gallery_search(menu, false),
            _ => return false,
        }
        true
    }

    /// The gallery's answers, applied to the state that asked for them; a
    /// query typed in the search box starts a search after a pause.
    pub(super) fn poll_pack_gallery(&mut self, menu: bool) {
        let due = {
            let state = &self.gallery_ui_mut(menu).gallery;
            state.changed_at.is_some_and(|at| at.elapsed() >= DEBOUNCE)
                && state.query != state.searched
        };
        if due {
            self.gallery_ui_mut(menu).gallery.changed_at = None;
            self.start_gallery_search(menu, false);
        }
        let mut msgs = Vec::new();
        while let Ok(msg) = self.pack_gallery.rx.try_recv() {
            msgs.push(msg);
        }
        for msg in msgs {
            self.apply_pack_gallery(msg);
        }
    }

    fn apply_pack_gallery(&mut self, msg: Msg) {
        match msg {
            Msg::Search {
                menu,
                generation,
                offset,
                query,
                result,
            } => {
                if generation != self.pack_gallery.generations[slot(menu)] {
                    return;
                }
                let state = &mut self.gallery_ui_mut(menu).gallery;
                match result {
                    Ok((found, total)) => {
                        if offset == 0 {
                            state.results = found;
                        } else {
                            state.results.extend(found);
                        }
                        state.total = total;
                        state.searched = query;
                        state.error.clear();
                    }
                    Err(error) => {
                        log::warn!("Horizon: the Modrinth search failed: {error}");
                        state.error = error;
                    }
                }
                state.status = gallery::Status::Ready;
            }
            Msg::Progress {
                menu,
                project_id,
                done,
                total,
            } => {
                self.gallery_ui_mut(menu)
                    .gallery
                    .downloading
                    .insert(project_id, (done, total));
            }
            Msg::Installed {
                menu,
                record,
                replace,
            } => self.finish_gallery_install(menu, record, replace),
            Msg::InstallFailed {
                menu,
                project_id,
                error,
            } => {
                log::warn!("Horizon: a gallery download failed: {error}");
                let state = &mut self.gallery_ui_mut(menu).gallery;
                state.downloading.remove(&project_id);
                state.warning = error;
            }
            Msg::Updates { menu, updates } => {
                self.gallery_ui_mut(menu).gallery.updates =
                    updates.into_iter().map(|u| (u.file.clone(), u)).collect();
            }
        }
    }

    /// The gallery page opened: read the sidecar, look for updates and, with
    /// no results yet, search.
    fn list_pack_gallery(&mut self, menu: bool) {
        let dir = packs::packs_dir();
        let records: Vec<gallery::Installed> = read_index()
            .into_iter()
            .filter(|record| dir.join(&record.file).is_file())
            .collect();
        {
            let state = &mut self.gallery_ui_mut(menu).gallery;
            state.installed = records
                .iter()
                .map(|record| (record.file.clone(), record.clone()))
                .collect();
        }
        if !records.is_empty()
            && self
                .pack_gallery
                .checked
                .is_none_or(|at| at.elapsed() >= UPDATE_EVERY)
        {
            self.pack_gallery.checked = Some(Instant::now());
            self.gallery_ui_mut(menu).gallery.checked_at = Some(Instant::now());
            let tx = self.pack_gallery.tx.clone();
            let _ = std::thread::Builder::new()
                .name("pack-gallery-updates".into())
                .spawn(move || check_updates(tx, menu, records));
        }
        let fresh = {
            let state = &self.gallery_ui_mut(menu).gallery;
            state.results.is_empty() && state.status != gallery::Status::Loading
        };
        if fresh {
            self.start_gallery_search(menu, false);
        }
    }

    /// Ask Modrinth for `query`'s page. The running search's number moves on:
    /// a page of an older one that arrives is dropped.
    fn start_gallery_search(&mut self, menu: bool, more: bool) {
        let (query, offset) = {
            let state = &mut self.gallery_ui_mut(menu).gallery;
            if !more {
                state.results.clear();
                state.total = 0;
                state.error.clear();
            }
            state.status = if more {
                gallery::Status::More
            } else {
                gallery::Status::Loading
            };
            let offset = if more { state.results.len() } else { 0 };
            (state.query.clone(), offset)
        };
        let index = slot(menu);
        self.pack_gallery.generations[index] += 1;
        let generation = self.pack_gallery.generations[index];
        let tx = self.pack_gallery.tx.clone();
        let _ = std::thread::Builder::new()
            .name("pack-gallery".into())
            .spawn(move || {
                let result = search(&query, offset);
                let _ = tx.send(Msg::Search {
                    menu,
                    generation,
                    offset,
                    query,
                    result,
                });
            });
    }

    /// Install gallery result `index` of the page.
    fn install_gallery_result(&mut self, menu: bool, index: usize) {
        let Some(pack) = self.gallery_ui_mut(menu).gallery.results.get(index).cloned() else {
            return;
        };
        if self
            .gallery_ui_mut(menu)
            .gallery
            .downloading
            .contains_key(&pack.project_id)
        {
            return;
        }
        if pack.latest.is_empty() {
            self.gallery_ui_mut(menu).gallery.warning =
                horizon_client::lang::get_or(
                    "horizon.pack_gallery.no_version",
                    "This pack has no downloadable version",
                );
            return;
        }
        self.gallery_ui_mut(menu)
            .gallery
            .downloading
            .insert(pack.project_id.clone(), (0, 0));
        let tx = self.pack_gallery.tx.clone();
        let project_id = pack.project_id.clone();
        let title = pack.title.clone();
        let latest = pack.latest.clone();
        let _ = std::thread::Builder::new()
            .name("pack-gallery-install".into())
            .spawn(move || run_install(tx, menu, project_id, title, fetch_version(&latest), None));
    }

    /// Install the update an installed pack has.
    fn install_gallery_update(&mut self, menu: bool, local: &str) {
        let Some(update) = self
            .gallery_ui_mut(menu)
            .gallery
            .updates
            .get(local)
            .cloned()
        else {
            return;
        };
        if self
            .gallery_ui_mut(menu)
            .gallery
            .downloading
            .contains_key(&update.project_id)
        {
            return;
        }
        self.gallery_ui_mut(menu)
            .gallery
            .downloading
            .insert(update.project_id.clone(), (0, update.size));
        let file = gallery::FileRef {
            version_id: update.version_id.clone(),
            version_name: update.version_name.clone(),
            file_name: update.file_name.clone(),
            url: update.url.clone(),
            sha512: update.sha512.clone(),
            size: update.size,
        };
        let tx = self.pack_gallery.tx.clone();
        let project_id = update.project_id.clone();
        let title = update.title.clone();
        let replace = Some(local.to_owned());
        let _ = std::thread::Builder::new()
            .name("pack-gallery-install".into())
            .spawn(move || run_install(tx, menu, project_id, title, Ok(file), replace));
    }

    /// A download landed in `resourcepacks/`: the index keeps it, the old
    /// file of an update goes, and the pack joins the selection.
    fn finish_gallery_install(
        &mut self,
        menu: bool,
        record: gallery::Installed,
        replace: Option<String>,
    ) {
        let installed: Vec<gallery::Installed> = {
            let state = &mut self.gallery_ui_mut(menu).gallery;
            state.downloading.remove(&record.project_id);
            state.warning.clear();
            if let Some(old) = &replace {
                state.installed.remove(old);
                state.updates.remove(old);
            }
            state.installed.insert(record.file.clone(), record.clone());
            state.installed.values().cloned().collect()
        };
        write_index(&installed);
        if let Some(old) = &replace
            && old != &record.file
        {
            let _ = std::fs::remove_file(packs::packs_dir().join(old));
        }
        self.enable_gallery_pack(menu, &record, replace.as_deref());
    }

    /// Auto-enable: the pack joins the player's selection (an update keeps the
    /// place the file it replaces had), and both pack screens are listed
    /// again with it.
    fn enable_gallery_pack(
        &mut self,
        menu: bool,
        record: &gallery::Installed,
        replace: Option<&str>,
    ) {
        let new_id = format!("file/{}", record.file);
        let mut selection = {
            let ui = self.gallery_ui_mut(menu);
            // What the player had picked on the screen so far is kept.
            if let Some(screen) = &mut ui.pack_screen
                && screen.changed
            {
                ui.pack_selection = screen.commit();
                screen.changed = false;
            }
            ui.pack_selection.clone()
        };
        let mut changed = false;
        match replace {
            Some(old) => {
                let old_id = format!("file/{old}");
                if old_id != new_id
                    && !selection.contains(&new_id)
                    && let Some(i) = selection.iter().position(|id| *id == old_id)
                {
                    selection[i] = new_id.clone();
                    changed = true;
                }
            }
            None => {
                if !selection.contains(&new_id) {
                    selection.push(new_id.clone());
                    changed = true;
                }
            }
        }
        if changed {
            self.settings.resource_packs = selection.clone();
            self.settings.save(&Self::options_path());
        }
        let infos = super::packs::pack_infos(&self.vanilla_root(), &self.server_packs.active);
        for which in [false, true] {
            let ui = self.gallery_ui_mut(which);
            ui.update_pack_infos(infos.clone());
            if changed {
                ui.pack_selection = selection.clone();
                ui.reselect();
            }
        }
    }
}

/// A search page from Modrinth.
fn search(query: &str, offset: usize) -> Result<(Vec<gallery::Pack>, u64), String> {
    let text = request(&gallery::search_url(query, offset))?;
    gallery::parse_search(&text)
}

/// The file of Modrinth version `id`.
fn fetch_version(id: &str) -> Result<gallery::FileRef, String> {
    let text = request(&format!("https://api.modrinth.com/v2/version/{id}"))?;
    gallery::parse_version(&text)
}

/// The newer version of every installed pack, for the gallery's update check.
fn check_updates(tx: Sender<Msg>, menu: bool, installed: Vec<gallery::Installed>) {
    let mut updates = Vec::new();
    for record in installed {
        let url = format!(
            "https://api.modrinth.com/v2/project/{}/version",
            record.project_id
        );
        match request(&url).and_then(|text| {
            gallery::parse_project_versions(&text, &record.version_id)
        }) {
            Ok(Some(file)) => updates.push(gallery::Update {
                file: record.file.clone(),
                title: record.title.clone(),
                project_id: record.project_id.clone(),
                version_id: file.version_id,
                version_name: file.version_name,
                file_name: file.file_name,
                url: file.url,
                sha512: file.sha512,
                size: file.size,
            }),
            Ok(None) => {}
            Err(error) => log::warn!(
                "Horizon: could not check {} for updates: {error}",
                record.project_id
            ),
        }
    }
    let _ = tx.send(Msg::Updates { menu, updates });
}

/// The whole install of one pack: resolve, download, check and place.
fn run_install(
    tx: Sender<Msg>,
    menu: bool,
    project_id: String,
    title: String,
    file: Result<gallery::FileRef, String>,
    replace: Option<String>,
) {
    let result = file.and_then(|file| {
        let part = download(&tx, menu, &project_id, &file)?;
        let stored = place(&part, &file.file_name)?;
        Ok((stored, file.version_id))
    });
    match result {
        Ok((stored, version_id)) => {
            let _ = tx.send(Msg::Installed {
                menu,
                record: gallery::Installed {
                    file: stored,
                    title,
                    project_id,
                    version_id,
                },
                replace,
            });
        }
        Err(error) => {
            let _ = tx.send(Msg::InstallFailed {
                menu,
                project_id,
                error,
            });
        }
    }
}

/// The file into the cache, its SHA-512 checked when Modrinth gave one.
fn download(
    tx: &Sender<Msg>,
    menu: bool,
    project_id: &str,
    file: &gallery::FileRef,
) -> Result<PathBuf, String> {
    let dir = horizon_client::logging::cache_dir().join("pack-gallery");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let part = dir.join(format!("{}.part", sanitize(stem_of(&file.file_name))));
    let response = agent()
        .get(&file.url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Modrinth: {e}"))?;
    let mut body = response.into_reader();
    let mut out = std::fs::File::create(&part).map_err(|e| format!("{}: {e}", part.display()))?;
    let mut done = 0u64;
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = body.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        done += read as u64;
        out.write_all(&buffer[..read]).map_err(|e| e.to_string())?;
        let _ = tx.send(Msg::Progress {
            menu,
            project_id: project_id.to_owned(),
            done,
            total: file.size,
        });
    }
    out.flush().map_err(|e| e.to_string())?;
    drop(out);
    let actual = sha512_of(&part)?;
    if !file.sha512.is_empty() && actual != file.sha512 {
        let _ = std::fs::remove_file(&part);
        return Err("the file does not match its SHA-512".to_owned());
    }
    Ok(part)
}

/// The downloaded file into `resourcepacks/`, under a name that is free.
fn place(part: &Path, file_name: &str) -> Result<String, String> {
    let dir = packs::packs_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let stored = unique_name(&dir, file_name);
    std::fs::copy(part, dir.join(&stored)).map_err(|e| format!("{stored}: {e}"))?;
    let _ = std::fs::remove_file(part);
    Ok(stored)
}

/// `<stem>.zip`, ` (2)`, ` (3)`… until the name is free, as the importer names.
fn unique_name(dir: &Path, file_name: &str) -> String {
    let stem = sanitize(stem_of(file_name));
    let mut name = format!("{stem}.zip");
    let mut n = 2;
    while dir.join(&name).exists() {
        name = format!("{stem} ({n}).zip");
        n += 1;
    }
    name
}

/// The file's own name, without folders or the extension.
fn stem_of(file_name: &str) -> &str {
    let name = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    name.strip_suffix(".zip").unwrap_or(name)
}

/// A name the file system (and the pack screen) takes: no separators or
/// control characters.
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').trim();
    if trimmed.is_empty() {
        "pack".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn sha512_of(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha512};
    let mut hash = Sha512::new();
    let mut file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut buffer = vec![0u8; 256 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(30))
        .build()
}

fn request(url: &str) -> Result<String, String> {
    agent()
        .get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Modrinth: {e}"))?
        .into_string()
        .map_err(|e| format!("Modrinth: {e}"))
}

/// The game folder's `pack-gallery.json`.
fn index_path() -> PathBuf {
    Game::game_dir().join(INDEX)
}

/// What the gallery installed, from the sidecar (nothing when it is missing
/// or does not read).
fn read_index() -> Vec<gallery::Installed> {
    let Ok(text) = std::fs::read_to_string(index_path()) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    value["packs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|pack| {
            Some(gallery::Installed {
                file: pack["file"].as_str()?.to_owned(),
                title: pack["title"].as_str().unwrap_or_default().to_owned(),
                project_id: pack["project_id"].as_str()?.to_owned(),
                version_id: pack["version_id"].as_str().unwrap_or_default().to_owned(),
            })
        })
        .collect()
}

fn write_index(installed: &[gallery::Installed]) {
    let packs: Vec<serde_json::Value> = installed
        .iter()
        .map(|pack| {
            serde_json::json!({
                "file": pack.file,
                "title": pack.title,
                "project_id": pack.project_id,
                "version_id": pack.version_id,
            })
        })
        .collect();
    let text = serde_json::to_string_pretty(&serde_json::json!({ "packs": packs }));
    match text {
        Ok(text) => {
            if let Err(e) = std::fs::write(index_path(), text) {
                log::warn!("Horizon: the gallery index was not saved: {e}");
            }
        }
        Err(e) => log::warn!("Horizon: the gallery index did not write: {e}"),
    }
}

//! The Modrinth resource-pack gallery's workers and effects (the page, at
//! `Options > Resource Packs > Download Packs`): search and version requests,
//! a download per install with its SHA-512 checked, the sidecar of what was
//! installed (`pack-gallery.json`), the update check and the auto-enable.
//! [`GalleryOps`] works on any options UI, so the title screen's menu (a
//! `Loading`) and the game's own options both use it; the page itself draws
//! `horizon_client::pack_gallery::Gallery`.
use super::{Game, packs};
use horizon_client::options_ui::{Event, OptionsUi};
use horizon_client::pack_gallery as gallery;
use horizon_client::packs::Pack;
use horizon_client::settings::Settings;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock,
        mpsc::{Receiver, Sender},
    },
    time::{Duration, Instant},
};

/// How long after typing a search runs.
const DEBOUNCE: Duration = Duration::from_millis(450);
/// How long an update check's answer is trusted before it is asked again.
const UPDATE_EVERY: Duration = Duration::from_secs(60);
/// At most this many icon downloads at once: a page of results should not
/// open two dozen connections together on a phone.
const ICON_LANES: usize = 4;
/// An icon that failed (a dropped mobile connection, a rate limit) is fetched
/// again up to this many times.
const ICON_RETRIES: u8 = 3;
/// With at least this long between tries.
const ICON_RETRY_AFTER: Duration = Duration::from_secs(3);
/// Who Modrinth sees: it asks for the app and where to reach its makers.
const USER_AGENT: &str = "Rezyth-Launcher (github.com/johnrenonasuncion-Ramil/Rezyth-Launcher)";
/// The sidecar of what the gallery installed, in the game folder.
const INDEX: &str = "pack-gallery.json";

/// What a worker sends back; [`GalleryOps::poll`] applies it.
pub(super) enum Msg {
    Search {
        generation: u64,
        offset: usize,
        query: String,
        result: Result<(Vec<gallery::Pack>, u64), String>,
    },
    Progress {
        project_id: String,
        done: u64,
        total: u64,
    },
    Installed {
        record: gallery::Installed,
        replace: Option<String>,
    },
    InstallFailed {
        project_id: String,
        error: String,
    },
    Updates {
        updates: Vec<gallery::Update>,
    },
    /// A row's icon downloaded (`None`: it did not decode).
    Icon {
        project: String,
        pixels: Option<Arc<[u8]>>,
    },
}

/// One options UI's gallery workers: its channel, the running search's number
/// and when the installed packs were last checked.
pub(super) struct GalleryOps {
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    generation: u64,
    checked: Option<Instant>,
    /// The icon downloads running now.
    icons_in_flight: usize,
    /// The icons that failed: how many tries, and when the last one was.
    icons_failed: BTreeMap<String, (u8, Instant)>,
}

impl GalleryOps {
    pub(super) fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            tx,
            rx,
            generation: 0,
            checked: None,
            icons_in_flight: 0,
            icons_failed: BTreeMap::new(),
        }
    }
}

impl Default for GalleryOps {
    fn default() -> Self {
        Self::new()
    }
}

impl GalleryOps {
    /// What the gallery's events ask of `ui`; None: not a gallery event.
    /// `Some(true)` when the settings changed (the caller saves them).
    pub(super) fn event(
        &mut self,
        ui: &mut OptionsUi,
        settings: &mut Settings,
        root: &Path,
        server: &[Pack],
        event: &Event,
    ) -> Option<bool> {
        match event {
            Event::ListPackGallery => self.list(ui),
            Event::PackGalleryInstall(i) => self.install_result(ui, *i),
            Event::PackGalleryUpdate(file) => self.install_update(ui, file),
            Event::PackGalleryMore => self.search(ui, true),
            Event::PackGalleryRetry => self.search(ui, false),
            _ => return None,
        }
        let _ = (settings, root, server);
        Some(false)
    }

    /// The workers' answers each frame `ui` draws, and the search box's
    /// debounce. Whether the settings changed (the caller saves them).
    pub(super) fn poll(
        &mut self,
        ui: &mut OptionsUi,
        settings: &mut Settings,
        root: &Path,
        server: &[Pack],
    ) -> bool {
        let due = {
            let state = &ui.gallery;
            state.changed_at.is_some_and(|at| at.elapsed() >= DEBOUNCE)
                && state.query != state.searched
        };
        if due {
            ui.gallery.changed_at = None;
            self.search(ui, false);
        }
        let mut msgs = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            msgs.push(msg);
        }
        let mut changed = false;
        for msg in msgs {
            changed |= self.apply(msg, ui, settings, root, server);
        }
        self.request_icons(ui);
        changed
    }

    fn apply(
        &mut self,
        msg: Msg,
        ui: &mut OptionsUi,
        settings: &mut Settings,
        root: &Path,
        server: &[Pack],
    ) -> bool {
        match msg {
            Msg::Search {
                generation,
                offset,
                query,
                result,
            } => {
                if generation != self.generation {
                    return false;
                }
                match result {
                    Ok((found, total)) => {
                        if offset == 0 {
                            ui.gallery.results = found;
                            // A fresh search is also a fresh start for the
                            // icons whose tries had run out.
                            self.icons_failed.clear();
                        } else {
                            ui.gallery.results.extend(found);
                        }
                        ui.gallery.total = total;
                        ui.gallery.searched = query;
                        ui.gallery.error.clear();
                        refresh_icons(ui);
                    }
                    Err(error) => {
                        log::warn!("Horizon: the Modrinth search failed: {error}");
                        ui.gallery.error = error;
                    }
                }
                ui.gallery.status = gallery::Status::Ready;
                false
            }
            Msg::Progress {
                project_id,
                done,
                total,
            } => {
                ui.gallery.downloading.insert(project_id, (done, total));
                false
            }
            Msg::Installed { record, replace } => {
                finish(ui, settings, root, server, record, replace)
            }
            Msg::InstallFailed {
                project_id,
                error,
            } => {
                log::warn!("Horizon: a gallery download failed: {error}");
                ui.gallery.downloading.remove(&project_id);
                ui.gallery.warning = error;
                false
            }
            Msg::Updates { updates } => {
                ui.gallery.updates = updates.into_iter().map(|u| (u.file.clone(), u)).collect();
                false
            }
            Msg::Icon { project, pixels } => {
                self.icons_in_flight = self.icons_in_flight.saturating_sub(1);
                match pixels {
                    Some(pixels) if ui.gallery.icon_urls.contains_key(&project) => {
                        ui.gallery.icons.insert(project.clone(), pixels);
                        self.icons_failed.remove(&project);
                    }
                    // Downloaded, but no row shows it any more: let a later
                    // listing ask again.
                    Some(_) => {
                        ui.gallery.icons_requested.remove(&project);
                    }
                    None => {
                        // A dropped phone connection or a rate limit must not
                        // leave the row on the fallback for good: ask again
                        // until the tries run out.
                        ui.gallery.icons_requested.remove(&project);
                        let failed = self
                            .icons_failed
                            .entry(project.clone())
                            .or_insert((0, Instant::now()));
                        failed.0 = failed.0.saturating_add(1);
                        failed.1 = Instant::now();
                        if failed.0 >= ICON_RETRIES {
                            log::warn!(
                                "Horizon: the icon of {project} did not download in {} tries",
                                failed.0
                            );
                        }
                    }
                }
                false
            }
        }
    }

    /// The gallery page opened: read the sidecar, look for updates and, with
    /// no results yet, search.
    fn list(&mut self, ui: &mut OptionsUi) {
        let dir = packs::packs_dir();
        let records: Vec<gallery::Installed> = read_index()
            .into_iter()
            .filter(|record| dir.join(&record.file).is_file())
            .collect();
        ui.gallery.installed = records
            .iter()
            .map(|record| (record.file.clone(), record.clone()))
            .collect();
        refresh_icons(ui);
        if !records.is_empty()
            && self
                .checked
                .is_none_or(|at| at.elapsed() >= UPDATE_EVERY)
        {
            self.checked = Some(Instant::now());
            ui.gallery.checked_at = Some(Instant::now());
            let tx = self.tx.clone();
            let _ = std::thread::Builder::new()
                .name("pack-gallery-updates".into())
                .spawn(move || check_updates(tx, records));
        }
        let fresh =
            ui.gallery.results.is_empty() && ui.gallery.status != gallery::Status::Loading;
        if fresh {
            self.search(ui, false);
        }
    }

    /// Ask Modrinth for `query`'s page. The running search's number moves on:
    /// a page of an older one that arrives is dropped.
    fn search(&mut self, ui: &mut OptionsUi, more: bool) {
        let (query, offset) = {
            let state = &mut ui.gallery;
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
        self.generation += 1;
        let generation = self.generation;
        let tx = self.tx.clone();
        let _ = std::thread::Builder::new()
            .name("pack-gallery".into())
            .spawn(move || {
                let result = search(&query, offset);
                let _ = tx.send(Msg::Search {
                    generation,
                    offset,
                    query,
                    result,
                });
            });
    }

    /// Install gallery result `index` of the page.
    fn install_result(&mut self, ui: &mut OptionsUi, index: usize) {
        let Some(pack) = ui.gallery.results.get(index).cloned() else {
            return;
        };
        if ui.gallery.downloading.contains_key(&pack.project_id) {
            return;
        }
        if pack.latest.is_empty() {
            ui.gallery.warning = horizon_client::lang::get_or(
                "horizon.pack_gallery.no_version",
                "This pack has no downloadable version",
            );
            return;
        }
        ui.gallery
            .downloading
            .insert(pack.project_id.clone(), (0, 0));
        let tx = self.tx.clone();
        let project_id = pack.project_id.clone();
        let title = pack.title.clone();
        let latest = pack.latest.clone();
        let icon = pack.icon.clone();
        let _ = std::thread::Builder::new()
            .name("pack-gallery-install".into())
            .spawn(move || run_install(tx, project_id, title, icon, fetch_version(&latest), None));
    }

    /// Install the update an installed pack has.
    fn install_update(&mut self, ui: &mut OptionsUi, local: &str) {
        let Some(update) = ui.gallery.updates.get(local).cloned() else {
            return;
        };
        if ui.gallery.downloading.contains_key(&update.project_id) {
            return;
        }
        ui.gallery
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
        let tx = self.tx.clone();
        let project_id = update.project_id.clone();
        let title = update.title.clone();
        let icon = ui
            .gallery
            .installed
            .get(local)
            .and_then(|record| record.icon.clone())
            .or_else(|| ui.gallery.icon_urls.get(&update.project_id).cloned());
        let replace = Some(local.to_owned());
        let _ = std::thread::Builder::new()
            .name("pack-gallery-install".into())
            .spawn(move || run_install(tx, project_id, title, icon, Ok(file), replace));
    }

    /// Ask for the icons the rows show but do not have yet, a few at a time: a
    /// lane frees up as each answer arrives, so a page of results never opens
    /// twenty connections together. A failed icon is tried again a few times
    /// apart; after that it keeps the fallback until a new search lists it.
    fn request_icons(&mut self, ui: &mut OptionsUi) {
        self.icons_failed
            .retain(|project, _| ui.gallery.icon_urls.contains_key(project));
        if self.icons_in_flight >= ICON_LANES {
            return;
        }
        let now = Instant::now();
        let mut wanted = Vec::new();
        for (project, url) in &ui.gallery.icon_urls {
            if ui.gallery.icons.contains_key(project)
                || ui.gallery.icons_requested.contains(project)
            {
                continue;
            }
            if let Some((tries, at)) = self.icons_failed.get(project)
                && (*tries >= ICON_RETRIES || now.duration_since(*at) < ICON_RETRY_AFTER)
            {
                continue;
            }
            wanted.push((project.clone(), url.clone()));
        }
        for (project, url) in wanted
            .into_iter()
            .take(ICON_LANES - self.icons_in_flight)
        {
            ui.gallery.icons_requested.insert(project.clone());
            let tx = self.tx.clone();
            let asked = project.clone();
            let spawned = std::thread::Builder::new()
                .name("pack-gallery-icon".into())
                .spawn(move || {
                    let pixels = fetch_icon(&url);
                    if pixels.is_none() {
                        log::debug!("Horizon: the pack icon at {url} was not fetched");
                    }
                    let _ = tx.send(Msg::Icon {
                        project: asked,
                        pixels,
                    });
                });
            if spawned.is_ok() {
                self.icons_in_flight += 1;
            } else {
                // No thread: let the next frame ask again.
                ui.gallery.icons_requested.remove(&project);
            }
        }
    }
}

/// A download landed in `resourcepacks/`: the index keeps it, the old file of
/// an update goes, and the pack joins the selection; whether the settings
/// changed.
fn finish(
    ui: &mut OptionsUi,
    settings: &mut Settings,
    root: &Path,
    server: &[Pack],
    record: gallery::Installed,
    replace: Option<String>,
) -> bool {
    let installed: Vec<gallery::Installed> = {
        ui.gallery.downloading.remove(&record.project_id);
        ui.gallery.warning.clear();
        if let Some(old) = &replace {
            ui.gallery.installed.remove(old);
            ui.gallery.updates.remove(old);
        }
        ui.gallery.installed.insert(record.file.clone(), record.clone());
        ui.gallery.installed.values().cloned().collect()
    };
    write_index(&installed);
    if let Some(icon) = &record.icon {
        ui.gallery
            .icon_urls
            .insert(record.project_id.clone(), icon.clone());
    }
    if let Some(old) = &replace
        && old != &record.file
    {
        let _ = std::fs::remove_file(packs::packs_dir().join(old));
    }
    enable(ui, settings, root, server, &record, replace.as_deref())
}

/// Auto-enable: the pack joins the player's selection (an update keeps the
/// place the file it replaces had), and the open pack screen is listed again
/// with it.
fn enable(
    ui: &mut OptionsUi,
    settings: &mut Settings,
    root: &Path,
    server: &[Pack],
    record: &gallery::Installed,
    replace: Option<&str>,
) -> bool {
    let new_id = format!("file/{}", record.file);
    let mut selection = {
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
        settings.resource_packs = selection.clone();
    }
    ui.update_pack_infos(super::packs::pack_infos(root, server));
    if changed {
        ui.pack_selection = selection;
        ui.reselect();
    }
    changed
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
fn check_updates(tx: Sender<Msg>, installed: Vec<gallery::Installed>) {
    let mut updates = Vec::new();
    for record in installed {
        let url = format!(
            "https://api.modrinth.com/v2/project/{}/version",
            record.project_id
        );
        match request(&url)
            .and_then(|text| gallery::parse_project_versions(&text, &record.version_id))
        {
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
    let _ = tx.send(Msg::Updates { updates });
}

/// The whole install of one pack: resolve, download, check and place.
fn run_install(
    tx: Sender<Msg>,
    project_id: String,
    title: String,
    icon: Option<String>,
    file: Result<gallery::FileRef, String>,
    replace: Option<String>,
) {
    let result = file.and_then(|file| {
        let part = download(&tx, &project_id, &file)?;
        let stored = place(&part, &file.file_name)?;
        Ok((stored, file.version_id))
    });
    match result {
        Ok((stored, version_id)) => {
            let _ = tx.send(Msg::Installed {
                record: gallery::Installed {
                    file: stored,
                    title,
                    project_id,
                    version_id,
                    icon,
                },
                replace,
            });
        }
        Err(error) => {
            let _ = tx.send(Msg::InstallFailed {
                project_id,
                error,
            });
        }
    }
}

/// The file into the cache, its SHA-512 checked when Modrinth gave one.
fn download(
    tx: &Sender<Msg>,
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

/// Rebuild the rows' icon URLs from the results and the installed packs, and
/// drop the icons no row shows any more.
fn refresh_icons(ui: &mut OptionsUi) {
    let mut urls = std::collections::BTreeMap::new();
    for pack in &ui.gallery.results {
        if let Some(icon) = &pack.icon {
            urls.insert(pack.project_id.clone(), icon.clone());
        }
    }
    for record in ui.gallery.installed.values() {
        if let Some(icon) = &record.icon {
            urls.insert(record.project_id.clone(), icon.clone());
        }
    }
    ui.gallery.icon_urls = urls;
    let keep = ui.gallery.icon_urls.clone();
    ui.gallery.icons.retain(|project, _| keep.contains_key(project));
    ui.gallery
        .icons_requested
        .retain(|project| keep.contains_key(project));
}

/// The HTTP agent every request shares: its keep-alive connections save a TLS
/// handshake per request, which a page of icon downloads feels.
fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(15))
            .timeout_read(Duration::from_secs(30))
            .build()
    })
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

/// The most bytes an icon may take: Modrinth's largest are 1024x1024 PNGs of a
/// few hundred KiB, and a cap below that silently dropped those rows to the
/// fallback texture.
const MAX_ICON: u64 = 4 * 1024 * 1024;

/// A row's icon from Modrinth, decoded to the pack screen's square.
fn fetch_icon(url: &str) -> Option<Arc<[u8]>> {
    if !url.starts_with("https://") {
        return None;
    }
    let response = agent()
        .get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .ok()?;
    let mut body = response.into_reader();
    let mut data = Vec::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = body.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..read]);
        if data.len() as u64 > MAX_ICON {
            return None;
        }
    }
    horizon_client::pack_screen::load_icon(&data)
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
                icon: pack["icon"].as_str().map(str::to_owned),
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
                "icon": pack.icon,
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

impl Game<'_> {
    /// The gallery's events, on this window's options UI; None: not a gallery
    /// event. `Some(true)` when the settings changed (save them).
    pub(super) fn pack_gallery_event(&mut self, event: &Event) -> Option<bool> {
        let root = self.vanilla_root();
        let Game {
            pack_gallery,
            ui,
            settings,
            server_packs,
            ..
        } = self;
        pack_gallery.event(
            &mut ui.options,
            settings,
            &root,
            &server_packs.active,
            event,
        )
    }

    /// The gallery's answers each frame the options are drawn; whether the
    /// settings changed (save them).
    pub(super) fn poll_pack_gallery(&mut self) -> bool {
        let root = self.vanilla_root();
        let Game {
            pack_gallery,
            ui,
            settings,
            server_packs,
            ..
        } = self;
        pack_gallery.poll(&mut ui.options, settings, &root, &server_packs.active)
    }
}

//! The Modrinth resource-pack gallery (`Options > Resource Packs > Download
//! Packs`): the state the page draws, and the Modrinth answers it reads. The
//! network, the workers and the sidecar of what was installed are the game's
//! (`game/pack_gallery.rs`); this half never touches the disk.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Instant;

/// How many packs one search page holds (Modrinth's `limit`).
pub const PAGE: usize = 20;

/// The gallery's state, filled by the game (`game/pack_gallery.rs`).
#[derive(Clone, Debug, Default)]
pub struct Gallery {
    /// What is typed in the search box; the game starts a search after a
    /// pause (`changed_at`).
    pub query: String,
    /// The query the results are for.
    pub searched: String,
    /// When the typed query last changed.
    pub changed_at: Option<Instant>,
    /// The found packs, in Modrinth's order.
    pub results: Vec<Pack>,
    /// Modrinth's total hits for the query.
    pub total: u64,
    pub status: Status,
    /// A failed search: shown with a Try Again button.
    pub error: String,
    /// A failed install or update check.
    pub warning: String,
    /// The packs downloading now, by project id: the bytes so far and in all.
    pub downloading: BTreeMap<String, (u64, u64)>,
    /// The packs installed from the gallery, by their file in `resourcepacks/`.
    pub installed: BTreeMap<String, Installed>,
    /// The newer version of each installed pack, by its installed file.
    pub updates: BTreeMap<String, Update>,
    /// When the installed packs were checked for updates (the game's).
    pub checked_at: Option<Instant>,
    /// The icon URL of each row, by Modrinth project id (from a search or the
    /// installed packs' `pack-gallery.json`), which the game downloads
    /// (`game/pack_gallery.rs`).
    pub icon_urls: BTreeMap<String, String>,
    /// The icons downloaded so far, by project id: `PACK_ICON_SIZE` square
    /// RGBA, as the pack screen's (`pack_screen::load_icon`).
    pub icons: BTreeMap<String, Arc<[u8]>>,
    /// The projects whose icon was asked for already: a failed one is not
    /// asked again until a new search lists it.
    pub icons_requested: BTreeSet<String>,
}

/// Where the gallery's search is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Idle,
    Loading,
    /// The next page of the same query.
    More,
    Ready,
}

/// A pack a search found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pack {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub downloads: u64,
    /// The game versions the pack lists (`hit.versions`).
    pub versions: Vec<String>,
    /// Whether it lists the game's own version.
    pub compatible: bool,
    /// The Modrinth version id a search would install (`hit.latest_version`).
    pub latest: String,
    /// Its icon (`hit.icon_url`): the row shows it beside the name.
    pub icon: Option<String>,
}

/// A pack installed from the gallery.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Installed {
    /// Its file in `resourcepacks/`.
    pub file: String,
    pub title: String,
    pub project_id: String,
    pub version_id: String,
    /// The icon the row showed (`Pack.icon`), kept in the sidecar so update
    /// rows have one before a search.
    pub icon: Option<String>,
}

/// A newer version of an installed pack.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Update {
    /// The installed file the new version replaces.
    pub file: String,
    pub title: String,
    pub project_id: String,
    pub version_id: String,
    pub version_name: String,
    pub file_name: String,
    pub url: String,
    pub sha512: String,
    pub size: u64,
}

/// One file of a Modrinth version: what an install downloads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileRef {
    pub version_id: String,
    pub version_name: String,
    pub file_name: String,
    pub url: String,
    pub sha512: String,
    pub size: u64,
}

/// The URL of `query`'s page of resource packs.
pub fn search_url(query: &str, offset: usize) -> String {
    let facets = "[[\"project_type:resourcepack\"]]";
    format!(
        "https://api.modrinth.com/v2/search?query={}&facets={}&limit={PAGE}&offset={offset}",
        encode(query),
        encode(facets),
    )
}

/// `value` percent-encoded (RFC 3986's unreserved characters stay as they
/// are).
pub fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// A `/v2/search` answer: its packs and total.
pub fn parse_search(text: &str) -> Result<(Vec<Pack>, u64), String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Modrinth: {e}"))?;
    let hits = value["hits"]
        .as_array()
        .ok_or("Modrinth sent no results")?;
    let packs = hits.iter().filter_map(pack_of).collect();
    Ok((packs, value["total_hits"].as_u64().unwrap_or(0)))
}

fn pack_of(hit: &serde_json::Value) -> Option<Pack> {
    let project_id = hit["project_id"].as_str()?.to_owned();
    let versions: Vec<String> = hit["versions"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let compatible = versions
        .iter()
        .any(|v| v == horizon_core::version::VANILLA_VERSION);
    Some(Pack {
        project_id,
        slug: hit["slug"].as_str().unwrap_or_default().to_owned(),
        title: hit["title"].as_str().unwrap_or("Pack").to_owned(),
        author: hit["author"].as_str().unwrap_or_default().to_owned(),
        description: hit["description"].as_str().unwrap_or_default().to_owned(),
        downloads: hit["downloads"].as_u64().unwrap_or(0),
        versions,
        compatible,
        latest: hit["latest_version"].as_str().unwrap_or_default().to_owned(),
        icon: hit["icon_url"]
            .as_str()
            .filter(|url| url.starts_with("https://"))
            .map(str::to_owned),
    })
}

/// A `/v2/version/{id}` answer: the file to install.
pub fn parse_version(text: &str) -> Result<FileRef, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Modrinth: {e}"))?;
    file_of(&value).ok_or_else(|| "Modrinth sent no downloadable file".to_owned())
}

/// A `/v2/project/{id}/version` answer: the newest version when it is not
/// `installed` (by version id), for the update check. A version for the game's
/// own version wins over one for another.
pub fn parse_project_versions(text: &str, installed: &str) -> Result<Option<FileRef>, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Modrinth: {e}"))?;
    let list = value.as_array().ok_or("Modrinth sent no versions")?;
    let mut best: Option<(&serde_json::Value, bool)> = None;
    for version in list {
        let compatible = version["game_versions"]
            .as_array()
            .is_some_and(|list| {
                list.iter()
                    .any(|v| v.as_str() == Some(horizon_core::version::VANILLA_VERSION))
            });
        let better = match &best {
            None => true,
            Some((old, old_compatible)) => match (compatible, *old_compatible) {
                (true, false) => true,
                (false, true) => false,
                // Newer wins: the date's ISO order is the time's.
                _ => {
                    version["date_published"].as_str() > old["date_published"].as_str()
                }
            },
        };
        if better {
            best = Some((version, compatible));
        }
    }
    let (version, _) = best.ok_or("Modrinth sent no versions")?;
    if version["id"].as_str() == Some(installed) {
        return Ok(None);
    }
    Ok(file_of(version))
}

/// A version's primary file (else its first), for an install.
fn file_of(version: &serde_json::Value) -> Option<FileRef> {
    let files = version["files"].as_array()?;
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool() == Some(true))
        .or_else(|| files.first())?;
    let url = file["url"].as_str()?;
    if !url.starts_with("https://") {
        return None;
    }
    Some(FileRef {
        version_id: version["id"].as_str().unwrap_or_default().to_owned(),
        version_name: version["version_number"]
            .as_str()
            .or_else(|| version["name"].as_str())
            .unwrap_or_default()
            .to_owned(),
        file_name: file["filename"].as_str().unwrap_or("pack.zip").to_owned(),
        url: url.to_owned(),
        sha512: file["hashes"]["sha512"].as_str().unwrap_or_default().to_owned(),
        size: file["size"].as_u64().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_answer_becomes_packs() {
        let text = r#"{
            "hits": [
                {
                    "project_id": "abc123",
                    "slug": "faithful",
                    "title": "Faithful 32x",
                    "author": "Vattic",
                    "description": "A higher resolution default look",
                    "downloads": 12345678,
                    "versions": ["1.21", "26.3"],
                    "latest_version": "ver1",
                    "icon_url": "https://cdn.modrinth.com/data/abc123/icon.png"
                },
                {
                    "project_id": "def456",
                    "slug": "old",
                    "title": "Old Pack",
                    "author": "Someone",
                    "versions": ["1.20.1"],
                    "latest_version": "ver2",
                    "icon_url": ""
                }
            ],
            "total_hits": 42
        }"#;
        let (packs, total) = parse_search(text).unwrap();
        assert_eq!(total, 42);
        assert_eq!(packs.len(), 2);
        assert_eq!(packs[0].project_id, "abc123");
        assert_eq!(packs[0].title, "Faithful 32x");
        assert_eq!(packs[0].downloads, 12345678);
        assert_eq!(
            packs[0].icon.as_deref(),
            Some("https://cdn.modrinth.com/data/abc123/icon.png")
        );
        assert!(packs[0].compatible);
        assert_eq!(packs[0].latest, "ver1");
        assert!(!packs[1].compatible);
        // An empty or non-HTTPS icon is no icon: the row keeps its fallback.
        assert_eq!(packs[1].icon, None);
    }

    #[test]
    fn the_newest_version_chooses_the_update() {
        let text = r#"[
            {
                "id": "old", "version_number": "1.0", "date_published": "2026-01-01T00:00:00Z",
                "game_versions": ["26.3"],
                "files": [{"url": "https://cdn.modrinth.com/a.zip", "filename": "a.zip",
                           "hashes": {"sha512": "aa"}, "size": 10, "primary": true}]
            },
            {
                "id": "new", "version_number": "2.0", "date_published": "2026-02-01T00:00:00Z",
                "game_versions": ["26.3"],
                "files": [{"url": "https://cdn.modrinth.com/b.zip", "filename": "b.zip",
                           "hashes": {"sha512": "bb"}, "size": 20, "primary": true}]
            }
        ]"#;
        let file = parse_project_versions(text, "old").unwrap().unwrap();
        assert_eq!(file.version_id, "new");
        assert_eq!(file.version_name, "2.0");
        assert_eq!(file.file_name, "b.zip");
        assert_eq!(file.sha512, "bb");
        assert_eq!(file.size, 20);
        assert!(parse_project_versions(text, "new").unwrap().is_none());
    }

    #[test]
    fn a_version_for_the_game_wins_over_one_for_another() {
        let text = r#"[
            {
                "id": "other", "version_number": "1.0", "date_published": "2026-03-01T00:00:00Z",
                "game_versions": ["1.20.1"],
                "files": [{"url": "https://cdn.modrinth.com/c.zip", "filename": "c.zip",
                           "hashes": {"sha512": "cc"}, "size": 5, "primary": true}]
            },
            {
                "id": "ours", "version_number": "1.9", "date_published": "2026-01-01T00:00:00Z",
                "game_versions": ["26.3"],
                "files": [{"url": "https://cdn.modrinth.com/d.zip", "filename": "d.zip",
                           "hashes": {"sha512": "dd"}, "size": 5, "primary": true}]
            }
        ]"#;
        let file = parse_project_versions(text, "old").unwrap().unwrap();
        assert_eq!(file.version_id, "ours");
    }

    #[test]
    fn query_values_are_percent_encoded() {
        assert_eq!(encode("stay true & 32x"), "stay%20true%20%26%2032x");
        let url = search_url("a b", 20);
        assert!(url.contains("query=a%20b"));
        assert!(url.contains("project_type%3Aresourcepack"));
        assert!(url.contains("limit=20"));
        assert!(url.contains("offset=20"));
    }
}

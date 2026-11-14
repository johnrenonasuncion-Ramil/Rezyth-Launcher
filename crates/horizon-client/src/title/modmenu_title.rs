// Mod Menu, built into the game (it was the native mod `modmenu-1.1.2.hnmod`;
// see THIRD_PARTY_NOTICES.md): the title screen's side. The title lives in the
// client library, which can't reach the game's modules, so the game writes
// what the button needs here once a frame (`modmenu_glue::frame`) and reads
// the presses back; `title.rs` builds the Mods button into its own list from
// `STYLE`, where the mod replaced the whole list.
use super::{HudAtlas, HudBatch};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

/// The game is up: the startup screen shows the same title before it is, from
/// what the game kept last time (`load_startup`).
pub static READY: AtomicBool = AtomicBool::new(false);
/// The mods screen is over the title: the title isn't drawn.
pub static COVER: AtomicBool = AtomicBool::new(false);
/// The Mods button was pressed; the game opens the screen at its next frame.
pub static OPEN_REQUEST: AtomicBool = AtomicBool::new(false);
/// `mods_button_style`; `NONE` when `modify_title_screen` is off. The icon
/// (Mod Menu's own default) is the one `title.rs` shows by default.
pub static STYLE: AtomicU8 = AtomicU8::new(ICON);
pub const NONE: u8 = 255;
/// Below Realms: its own row, the rest moving 12 px apart around it.
pub(crate) const BELOW_REALMS: u8 = 0;
/// Replace Realms: where the (disabled) Realms button is.
pub(crate) const REPLACE_REALMS: u8 = 1;
/// Adjacent: Realms and Mods, 98 px each, on the Realms row.
pub(crate) const ADJACENT: u8 = 2;
/// Icon: a fifth 20x20 icon in the title's icon row. The default.
pub(crate) const ICON: u8 = 3;
/// The Mods button's id among the title's buttons.
pub const BUTTON_ID: u8 = 200;
/// Its text for the text styles ("Mods", "Mods (25 Loaded)").
pub static LABEL: Mutex<String> = Mutex::new(String::new());
/// What follows the title's version (" (25 Mods)"), or nothing.
pub static COUNT: Mutex<String> = Mutex::new(String::new());

/// `textures/gui/sprites/mods/enabled.png` (Mod Menu, MIT), 20x20: runs of
/// pixels (x, y, length, colour as the game packs it).
const MODS_ICON: &[(u8, u8, u8, u32)] = &[(4,4,10,0xffffffff),(15,4,1,0xffffffff),(5,5,10,0xff3f3f3f),(16,5,1,0xff3f3f3f),(4,6,2,0xffffffff),(7,6,9,0xffffffff),(4,7,2,0xffffffff),(6,7,1,0xff3f3f3f),(7,7,9,0xffffffff),(16,7,1,0xff3f3f3f),(5,8,2,0xff3f3f3f),(8,8,9,0xff3f3f3f),(4,9,2,0xffffffff),(7,9,9,0xffffffff),(4,10,2,0xffffffff),(6,10,1,0xff3f3f3f),(7,10,9,0xffffffff),(16,10,1,0xff3f3f3f),(5,11,2,0xff3f3f3f),(8,11,9,0xff3f3f3f),(4,12,2,0xffffffff),(7,12,9,0xffffffff),(4,13,2,0xffffffff),(6,13,1,0xff3f3f3f),(7,13,9,0xffffffff),(16,13,1,0xff3f3f3f),(5,14,2,0xff3f3f3f),(8,14,9,0xff3f3f3f),(4,15,5,0xffffffff),(11,15,5,0xffffffff),(5,16,5,0xff3f3f3f),(12,16,5,0xff3f3f3f)];

/// The Mods icon, 20 px wide.
pub fn draw_icon(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32) {
    for &(px, py, len, c) in MODS_ICON {
        batch.fill(atlas, x + f32::from(px), y + f32::from(py), f32::from(len), 1.0, c);
    }
}

/// Where the game keeps what the title last showed (the style, the button's
/// text and the count, a line each) for the startup screen, which shows the
/// title before the game is up: Mod Menu's data folder.
pub fn startup_path() -> std::path::PathBuf {
    let dir = std::env::current_dir().unwrap_or_default().join(crate::mod_list::DIR);
    dir.join(".horizon-mod-data").join("modmenu").join("startup.txt")
}

/// Before the game is up (the startup screen draws the title without the
/// game's frame running): what the title showed last time, read once.
pub(crate) fn ensure_startup() {
    if !READY.load(Ordering::Relaxed) {
        load_startup();
    }
}

/// What the title showed last time (`ensure_startup`).
fn load_startup() {
    static LOADED: OnceLock<()> = OnceLock::new();
    LOADED.get_or_init(|| {
        let Ok(text) = std::fs::read_to_string(startup_path()) else {
            return;
        };
        let mut lines = text.lines();
        if let Some(style) = lines.next().and_then(|l| l.trim().parse::<u8>().ok()) {
            STYLE.store(style, Ordering::Relaxed);
        }
        for slot in [&LABEL, &COUNT] {
            if let (Some(line), Ok(mut slot)) = (lines.next(), slot.lock())
                && slot.is_empty()
            {
                *slot = line.to_owned();
            }
        }
    });
}

/// A title button drawn, the startup screen's too: on the Mods button, its
/// icon (`SmallModMenuButtonWidget`) and the mod count after the version
/// (`MixinTitleScreen`).
pub(crate) fn drawn(b: &super::Button, batch: &mut HudBatch, atlas: &HudAtlas) {
    let (w, h) = (batch.width.ceil(), batch.height.ceil());
    if b.id != BUTTON_ID || button_rect(w, h) != Some((b.x, b.y, b.w)) {
        return;
    }
    if b.label.is_empty() {
        draw_icon(batch, atlas, b.x, b.y);
    }
    let count = COUNT.lock().map(|c| c.clone()).unwrap_or_default();
    if !count.is_empty() {
        let version = format!("Horizon {} (Rust)", horizon_core::VANILLA_VERSION);
        let x = (2.0 + HudBatch::text_width(atlas, &version)).floor();
        batch.text(atlas, &count, x, h - 10.0, u32::MAX, true);
    }
}

/// The Mods button's rectangle on the main title screen (x, y, w), for the
/// game's drawing of its icon and tooltip, and `drawn`'s check. One with
/// `title.rs`'s `buttons`, which builds the button for each style (the icon
/// row is five 20 px icons, 4 apart, centred: `left` = cx - 58).
pub fn button_rect(w: f32, h: f32) -> Option<(f32, f32, f32)> {
    let style = STYLE.load(Ordering::Relaxed);
    let (cx, y0) = ((w.ceil() / 2.0).floor(), (h.ceil() / 4.0).floor() + 48.0);
    match style {
        BELOW_REALMS => Some((cx - 100.0, y0 + 48.0 - 12.0 + 24.0, 200.0)),
        REPLACE_REALMS => Some((cx - 100.0, y0 + 48.0, 200.0)),
        ADJACENT => Some((cx + 2.0, y0 + 48.0, 98.0)),
        ICON => Some((cx - 58.0 + 4.0 * 24.0, y0 + 72.0, 20.0)),
        _ => None,
    }
}

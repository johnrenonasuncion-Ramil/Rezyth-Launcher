//! The look of the split-pane settings screens (`graphics_ui`,
//! `volt_settings_ui`, `volt_mods`), in one place: sizes and the controls they
//! draw with, styled as vanilla Minecraft's own widgets — the grey buttons,
//! sliders, text field, checkboxes and scrollbars, vanilla's text colours and
//! the tiled list background — while the screens keep their layout.

use crate::{
    hud::{HudAtlas, HudBatch, rgba},
    text::Text,
    ui,
};
use horizon_theme::BLOCKS_DARK as B;

/// What a setting costs in frame time, shown with its description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    None,
    Low,
    Medium,
    High,
    /// Turning it on makes frames faster (culling, caching, batching).
    Saves,
}
impl Cost {
    pub fn label(self) -> &'static str {
        match self {
            Cost::None => "No cost",
            Cost::Low => "Low cost",
            Cost::Medium => "Medium cost",
            Cost::High => "High cost",
            Cost::Saves => "Saves time",
        }
    }
    /// `0xRRGGBB`: the palette's success, warn and danger.
    pub fn rgb(self) -> u32 {
        match self {
            Cost::None => B.ink_soft,
            Cost::Low | Cost::Saves => B.success,
            Cost::Medium => B.warn,
            Cost::High => B.danger,
        }
    }
}

/// `0xRRGGBB` and an alpha in the HUD's colour encoding.
const fn hex(c: u32, a: u8) -> u32 {
    (a as u32) << 24 | (c & 0xff) << 16 | (c >> 8 & 0xff) << 8 | (c >> 16 & 0xff)
}
/// The extra ring over the search field at rest: the vanilla text field's own
/// sprite is its edge, so the ring draws none.
pub const EDGE: u32 = 0;
/// A hovered or focused row: vanilla's light wash over the list.
pub const HOVER: u32 = hex(0xffffff, 30);
/// The open page in the page list: the dark fill of a selected vanilla list
/// entry (the screens add its white outline).
pub const OPEN: u32 = hex(0x000000, 140);
/// The rule under the plate's heading, and between two rows of a group:
/// vanilla's separators are black.
pub const HEAD_RULE: u32 = hex(0x000000, 255);
pub const ROW_RULE: u32 = hex(0x000000, 200);
/// Vanilla has no accent colour; its highlights are white.
pub const ACCENT: u32 = u32::MAX;
/// Vanilla's white text, and its grey for what can't be used now.
pub const WHITE: u32 = u32::MAX;
pub const MUTED: u32 = hex(0xa0a0a0, 255);
/// A page that isn't open, a value at rest.
pub const SOFT: u32 = hex(0xe0e0e0, 255);
/// Vanilla's grey for a tooltip's lines under its first (`ChatFormatting.GRAY`).
pub const GREY: u32 = hex(0xaaaaaa, 255);
/// No veil: `ui::menu_background` already darkens the screen the vanilla way.
pub const VEIL: u32 = 0;

/// Sizes in GUI pixels.
/// Shorter screens than this (a 720p window at scale 3) keep a narrower margin on top.
pub const TITLED: f32 = 300.0;
/// A row is 24 px with a vanilla button's own 20 px height inside it.
pub const ROW: f32 = 24.0;
/// A group's name over its panel, and a page's over its groups while searching.
pub const GROUP_ROW: f32 = 14.0;
pub const PAGE_ROW: f32 = 16.0;
/// Space under a group's panel, and more before the next page's name.
pub const GROUP_GAP: f32 = 10.0;
pub const PAGE_GAP: f32 = 6.0;
/// The control on the right of a row (a button, a box or a slider), and the
/// left column: the search field, the page list and the buttons.
pub const CONTROL_W: f32 = 128.0;
/// Vanilla's buttons and sliders are 20 px tall: the row control is too.
pub const CONTROL_H: f32 = 20.0;
pub const SEARCH_H: f32 = 16.0;
/// Vanilla's scroller: the column `ui::Scroll::press` takes.
pub const SCROLLER_W: f32 = 6.0;
/// The strip under the plate that describes a setting.
pub const DOCK_H: f32 = 50.0;
pub const BUTTON_H: f32 = 20.0;
/// A row's name and control, from its group panel's edges.
pub const INSET: f32 = 10.0;

pub fn fill(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4], color: u32) {
    if w > 0.0 && h > 0.0 {
        batch.fill(atlas, x, y, w, h, color);
    }
}
/// A 1 px ring inside `rect`: the keyboard or controller focus, in white as vanilla draws it.
pub fn ring(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4], color: u32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    batch.fill(atlas, x, y, w, 1.0, color);
    batch.fill(atlas, x, y + h - 1.0, w, 1.0, color);
    if h > 2.0 {
        batch.fill(atlas, x, y + 1.0, 1.0, h - 2.0, color);
        batch.fill(atlas, x + w - 1.0, y + 1.0, 1.0, h - 2.0, color);
    }
}
/// The screen's plate, and the strip under it: vanilla's tiled list
/// background (`OptionsSubScreen`'s list, the world list).
pub fn panel(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4], in_world: bool) {
    let sprite = if in_world {
        "gui/inworld_menu_list_background"
    } else {
        "gui/menu_list_background"
    };
    if atlas.sprites.contains_key(sprite) {
        ui::tiled(batch, atlas, sprite, x, y, w, h, 32.0, u32::MAX);
    } else {
        fill(batch, atlas, [x, y, w, h], rgba(0, 0, 0, 120));
    }
    // A gentle wash for the text on the tiles, and a black frame, as vanilla's
    // containers have.
    fill(batch, atlas, [x, y, w, h], hex(0x000000, 60));
    ring(batch, atlas, [x, y, w, h], hex(0x000000, 200));
}
/// Vanilla's option lists have no group panels: the rows on the list
/// background, and their hover, are the whole look.
pub fn group(_batch: &mut HudBatch, _atlas: &HudAtlas, _rect: [f32; 4]) {}
/// A vanilla text field's background (the search box).
pub fn well(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4]) {
    ui::nine_slice(
        batch,
        atlas,
        "gui/sprites/widget/text_field",
        x,
        y,
        w,
        h,
        u32::MAX,
    );
}
/// The scroller: vanilla's track and thumb sprites.
pub fn scroller(batch: &mut HudBatch, atlas: &HudAtlas, track: [f32; 4], thumb: [f32; 4]) {
    ui::nine_slice(
        batch,
        atlas,
        "gui/sprites/widget/scroller_background",
        track[0],
        track[1],
        track[2],
        track[3],
        u32::MAX,
    );
    ui::nine_slice(
        batch,
        atlas,
        "gui/sprites/widget/scroller",
        thumb[0],
        thumb[1],
        thumb[2],
        thumb[3],
        u32::MAX,
    );
}
/// The scroller's thumb for a list of this content: its top and height, or
/// None when everything fits.
pub fn scrollbar_thumb(top: f32, bottom: f32, content: f32, scroll: f32) -> Option<(f32, f32)> {
    let view = bottom - top;
    if content <= view || view < 16.0 {
        return None;
    }
    let max_bar = view - 6.0;
    let bar = ((view * view) / content)
        .clamp(24.0_f32.min(max_bar), max_bar)
        .floor();
    let max_scroll = (content - view).max(1.0);
    let ratio = (scroll / max_scroll).clamp(0.0, 1.0);
    let y = top + ((view - bar) * ratio).floor();
    Some((y, bar))
}
/// `text` cut to `width` with an ellipsis, as a long name gives way to its control.
pub fn fit(atlas: &HudAtlas, text: &str, width: f32) -> String {
    if width <= 8.0 {
        return String::new();
    }
    if atlas.font.width(text) <= width {
        return text.to_owned();
    }
    let mut out = String::new();
    for c in text.chars() {
        if atlas.font.width(&format!("{out}{c}...")) > width {
            break;
        }
        out.push(c);
    }
    format!("{}...", out.trim_end())
}
/// `text` broken at spaces to fit `width` (the Volt Settings description dock).
pub fn wrap_lines(atlas: &HudAtlas, text: &str, width: f32) -> Vec<String> {
    let room = width.max(40.0);
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let cur = lines.last().cloned().unwrap_or_default();
        let cand = if cur.is_empty() {
            word.to_owned()
        } else {
            format!("{cur} {word}")
        };
        if atlas.font.width(&cand) <= room || cur.is_empty() {
            if let Some(last) = lines.last_mut() {
                *last = cand;
            }
        } else {
            lines.push(word.to_owned());
        }
    }
    lines
}
/// Vanilla's checkbox: its 17 px sprite at the right of the row at `y`,
/// highlighted while `hover`ed.
pub fn checkbox(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    right: f32,
    y: f32,
    on: bool,
    live: bool,
    hover: bool,
) {
    let sprite = match (on, hover) {
        (true, true) => "gui/sprites/widget/checkbox_selected_highlighted",
        (true, false) => "gui/sprites/widget/checkbox_selected",
        (false, true) => "gui/sprites/widget/checkbox_highlighted",
        (false, false) => "gui/sprites/widget/checkbox",
    };
    let color = if live { u32::MAX } else { MUTED };
    let cy = y + (ROW - 17.0) / 2.0;
    batch.sprite_scaled(atlas, sprite, right - 17.0, cy, 17.0, 17.0, color);
}
/// A vanilla button: the grey widget, its label centred and scrolling when it
/// is wider than the button. `primary` is kept for the callers' tuple; vanilla
/// has one button look.
pub fn button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    [x, y, w, h]: [f32; 4],
    label: &Text,
    (hover, live, _primary): (bool, bool, bool),
) {
    let sprite = if !live {
        "gui/sprites/widget/button_disabled"
    } else if hover {
        "gui/sprites/widget/button_highlighted"
    } else {
        "gui/sprites/widget/button"
    };
    let sprite = ui::button_take(sprite, x.round() as i32, y.round() as i32);
    ui::nine_slice(batch, atlas, &sprite, x, y, w, h, u32::MAX);
    let color = if live { WHITE } else { MUTED };
    let ty = y + ((h - 8.0) / 2.0).floor();
    ui::scrolling_text(
        batch,
        atlas,
        label,
        x + 2.0,
        x + w - 2.0,
        y,
        y + h,
        ty,
        color,
    );
}
/// The ON/OFF row control is a vanilla checkbox (the pill becomes the sprite,
/// centred on the box it used to fill).
pub fn switch_pill(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    [x, y, w, h]: [f32; 4],
    _anim: f32,
    on: bool,
) {
    let sprite = if on {
        "gui/sprites/widget/checkbox_selected"
    } else {
        "gui/sprites/widget/checkbox"
    };
    let sx = x + w - 17.0;
    let sy = (y + h / 2.0 - 8.5).floor();
    batch.sprite_scaled(atlas, sprite, sx, sy, 17.0, 17.0, u32::MAX);
}
/// Undo is a vanilla button, like Done.
pub fn undo_button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    rect: [f32; 4],
    label: &Text,
    hover: bool,
) {
    button(batch, atlas, rect, label, (hover, true, false));
}
/// The split-pane screens' buttons, vanilla's own.
pub fn volt_button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    rect: [f32; 4],
    label: &Text,
    (hover, live, primary): (bool, bool, bool),
) {
    button(batch, atlas, rect, label, (hover, live, primary));
}
/// A vanilla slider — its 20 px track and handle over the row's control, the
/// value centred on it.
pub fn slider(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    [x, y, w, h]: [f32; 4],
    value: &Text,
    position: f32,
    (hover, live): (bool, bool),
) {
    // The vanilla slider keeps its 20 px height: centred where the row's
    // 16 px control sits.
    let sy = (y + h / 2.0 - 10.0).floor();
    let handle = if live && hover {
        "gui/sprites/widget/slider_handle_highlighted"
    } else {
        "gui/sprites/widget/slider_handle"
    };
    ui::nine_slice(
        batch,
        atlas,
        "gui/sprites/widget/slider",
        x,
        sy,
        w,
        20.0,
        u32::MAX,
    );
    let hx = x + (position.clamp(0.0, 1.0) * (w - 8.0)).floor();
    ui::nine_slice(batch, atlas, handle, hx, sy, 8.0, 20.0, u32::MAX);
    let color = if live { WHITE } else { MUTED };
    ui::centered_text(batch, atlas, value, x, sy + 6.0, w, color);
}
/// A cycle button is a vanilla button whose label is the value between its
/// arrows, fitted so a long value never scrolls.
pub fn cycler(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    [x, y, w, h]: [f32; 4],
    value: &Text,
    (hover, live): (bool, bool),
) {
    let sy = (y + h / 2.0 - 10.0).floor();
    let label = format!("< {} >", value.plain());
    let label = Text::literal(fit(atlas, &label, w - 6.0));
    button(batch, atlas, [x, sy, w, 20.0], &label, (hover, live, false));
}
/// A screen's title, centred at the top (split-screen profiles).
pub fn title(batch: &mut HudBatch, atlas: &HudAtlas, text: &Text) {
    let w = batch.width;
    ui::centered_text(batch, atlas, text, 0.0, 12.0, w, u32::MAX);
}
/// The cost's colour in the HUD's encoding: the square beside its name.
pub fn cost_color(cost: Cost) -> u32 {
    hex(cost.rgb(), 255)
}

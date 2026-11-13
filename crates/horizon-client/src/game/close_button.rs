//! The close button every container screen carries: a small vanilla button
//! in the frame's top-right corner, inside it — the place Bedrock draws its
//! X. A left press closes the screen as Escape does; a right press is
//! swallowed, as any widget swallows its own. The screens that lay
//! themselves out (the command block, the game master blocks, a book view)
//! draw it at their own corner through `draw_close_rect`.
use super::{Game, Screen, stations::Kind};
use horizon_client::{hud::HudBatch, lang, text::Text, ui};

/// The button's side in GUI pixels: small enough to stay above the first
/// slot row (y = 18) every frame's top-right column starts at.
const CLOSE: f32 = 14.0;
/// Its inset from the frame's edges.
const INSET: f32 = 4.0;

/// The close button's rect inside a frame `w` wide at (`x`, `y`): the
/// frame's top-right corner, inside it.
pub(super) fn rect(x: f32, y: f32, w: f32) -> [f32; 4] {
    [x + w - INSET - CLOSE, y + INSET, CLOSE, CLOSE]
}

/// What a press at the close button means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Press {
    /// The press missed the button: the screen handles it as before.
    Miss,
    /// The press hit the button: close the screen (Escape's own close).
    Close,
    /// The press hit it, but is not the button's own (a right press):
    /// swallowed, as a widget swallows its presses, so it reaches no slot
    /// and drops nothing outside.
    Swallow,
}

/// A press at `cursor` on the close button at `rect`.
pub(super) fn press(cursor: (f32, f32), rect: [f32; 4], right: bool) -> Press {
    if !ui::hovered(cursor, rect[0], rect[1], rect[2], rect[3]) {
        return Press::Miss;
    }
    if right { Press::Swallow } else { Press::Close }
}

impl Game<'_> {
    /// The close button's rect on the open container screen, from the
    /// screen's own frame. None for the screens that place their own
    /// widgets (a native mod's): they keep their own corners.
    pub(super) fn container_close_rect(&self, batch: &HudBatch) -> Option<[f32; 4]> {
        if self.screen != Screen::Inventory || self.native_screen_shown() {
            return None;
        }
        let (ox, oy) = self.inventory_origin(batch);
        // The wider frames share the 176-px origin, drawn centred around it
        // by their own shift (game/beacon.rs, game/trading.rs). A merchant
        // station before its trade state is set draws as the plain 176 menu
        // it then is (`draw_merchant` returns false).
        let station = self.station.as_ref();
        let (shift, width) = if station.is_some_and(|s| s.kind == Kind::Beacon) {
            (super::beacon::SHIFT, super::beacon::WIDTH)
        } else if station.is_some_and(|s| s.kind == Kind::Merchant && s.merchant.is_some()) {
            (super::trading::SHIFT, super::trading::WIDTH)
        } else {
            (0.0, 176.0)
        };
        Some(rect(ox + shift, oy, width))
    }
    /// Draw the close button of the open container screen, when it has one.
    pub(super) fn draw_close_button(&self, batch: &mut HudBatch) {
        if let Some(rect) = self.container_close_rect(batch) {
            self.draw_close_rect(batch, rect);
        }
    }
    /// Draw a close button over `rect`: a vanilla button with its "x",
    /// highlighted under the cursor. For the screens that carry the rect
    /// themselves (the command block, the game master blocks, a book).
    pub(super) fn draw_close_rect(&self, batch: &mut HudBatch, rect: [f32; 4]) {
        let hover = ui::hovered(self.cursor, rect[0], rect[1], rect[2], rect[3]);
        // The label is the glyph; the narration says what it does.
        let label = Text::literal("x");
        ui::button_as(
            batch,
            &self.gui.atlas,
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            &label,
            true,
            hover,
            |r| {
                horizon_client::widgets::Narratable::button(
                    lang::get_or("gui.close", "Close"),
                    r,
                    true,
                )
            },
        );
    }
    /// Whether the cursor is over `rect`: the hit test of the screens that
    /// carry their close rect themselves.
    pub(super) fn close_rect_hit(&self, rect: [f32; 4]) -> bool {
        ui::hovered(self.cursor, rect[0], rect[1], rect[2], rect[3])
    }
    /// A press on the open container screen's close button: close for a
    /// left press, swallow a right one, miss when not on it. The close is
    /// Escape's own for this screen (`key_pressed`), so the paths stay
    /// alike.
    pub(super) fn close_button_press(&mut self, right: bool) -> bool {
        let Some(size) = self
            .renderer
            .as_ref()
            .map(|r| [r.config.width, r.config.height])
        else {
            return false;
        };
        let batch = HudBatch::new(size);
        let Some(rect) = self.container_close_rect(&batch) else {
            return false;
        };
        match press(self.cursor, rect, right) {
            Press::Miss => false,
            Press::Swallow => {
                // The release must not click a slot (or the outside).
                self.inv_clicks.skip_release();
                true
            }
            Press::Close => {
                self.audio.play_button_click();
                // `close_container` cancels the screen's clicks itself.
                self.close_container();
                self.pause_focus = None;
                self.screen = Screen::Playing;
                self.pause_menu = true;
                self.capture(true);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Press, press, rect};

    #[test]
    fn the_close_button_sits_inside_the_frames_top_right_corner() {
        for (x, y, w) in [(0.0, 0.0, 176.0), (20.0, 10.0, 230.0), (-50.0, 0.0, 276.0)] {
            let [bx, by, bw, bh] = rect(x, y, w);
            assert!(bx >= x && bx + bw <= x + w, "inside the frame's width");
            assert!(by >= y && by + bh <= y + 20.0, "inside its top row");
            assert!(bx >= x + w / 2.0, "on its right half");
        }
    }

    #[test]
    fn a_left_press_closes_and_a_right_press_is_only_swallowed() {
        let r = rect(0.0, 0.0, 176.0);
        let on = (r[0] + 1.0, r[1] + 1.0);
        assert_eq!(press(on, r, false), Press::Close);
        assert_eq!(press(on, r, true), Press::Swallow);
        for miss in [
            (r[0] - 1.0, r[1] + 1.0),
            (r[0] + 1.0, r[1] - 1.0),
            (r[0] + r[2], r[1] + 1.0),
            (r[0] + 1.0, r[1] + r[3]),
        ] {
            assert_eq!(press(miss, r, false), Press::Miss, "{miss:?}");
        }
    }
}

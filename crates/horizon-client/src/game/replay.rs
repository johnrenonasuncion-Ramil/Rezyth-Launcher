//! Rezyth Settings' replay recorder (`volt_qol.replay_buffer`): the floating
//! button's action (or the `key.volt.replay` key, or a touch button bound to
//! it) cycles idle → recording ⇄ paused → idle, sampling the camera once per
//! world tick while recording and writing the track as JSON to the player's
//! `replays/` folder on the way out. The launcher-side player (timeline,
//! speeds, free camera) reads these files and is not built yet; the takes are
//! real, complete camera tracks.

use super::Game;
use horizon_client::volt_features::{ReplayState, ReplayTrack};
use horizon_client::{hud::HudBatch, logging, text::Text};
use std::time::Duration;
use winit::keyboard::KeyCode;

/// The replay button's held finger: its id, where it landed, when, and the
/// drag's mode (0 deciding, 1 move, 2 size/opacity) with where the long press
/// started and the values it started from.
#[derive(Clone, Copy)]
pub(super) struct ReplayTouch {
    pub(super) finger: u64,
    pub(super) start: [f32; 2],
    pub(super) began: std::time::Instant,
    pub(super) mode: u8,
    pub(super) mode_start: [f32; 2],
    pub(super) size: u16,
    pub(super) opacity: u16,
}

/// A drag decides once the finger moves 12 HUD units, or holds still for half
/// a second.
const DRAG_START: f32 = 12.0;
const HOLD: Duration = Duration::from_millis(500);

impl Game<'_> {
    /// The replay button's tap: start, pause, or end (writes the file).
    pub(super) fn replay_action(&mut self) {
        if !self.settings.extras.volt_qol.replay_buffer {
            return;
        }
        match self.replay_state {
            ReplayState::Idle => {
                self.replay_track = Some(ReplayTrack {
                    dimension: self.dim.id().to_owned(),
                    start_game_time: self.world.game_time,
                    samples: Vec::new(),
                });
                self.replay_state = ReplayState::Recording;
                self.system_message(Text::translate("horizon.replay.started", &[]));
            }
            ReplayState::Recording => {
                self.replay_state = ReplayState::Paused;
                self.system_message(Text::translate("horizon.replay.paused", &[]));
            }
            ReplayState::Paused => {
                if let Some(track) = self.replay_track.take() {
                    self.save_replay(&track);
                }
                self.replay_state = ReplayState::Idle;
            }
        }
    }

    /// Write a finished take to the player's `replays/` folder and report it
    /// in chat (like a screenshot's message).
    fn save_replay(&mut self, track: &ReplayTrack) {
        let dir = logging::player_dir().join("replays");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.system_message(Text::translate(
                "horizon.replay.failed",
                &[Text::literal(e.to_string())],
            ));
            return;
        }
        let unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let name = horizon_client::volt_features::replay_file_name(unix, &track.dimension);
        let write = serde_json::to_string_pretty(track)
            .map_err(|e| e.to_string())
            .and_then(|json| std::fs::write(dir.join(&name), json).map_err(|e| e.to_string()));
        match write {
            Ok(()) => {
                self.system_message(Text::translate(
                    "horizon.replay.saved",
                    &[Text::literal(name)],
                ));
            }
            Err(e) => {
                self.system_message(Text::translate(
                    "horizon.replay.failed",
                    &[Text::literal(e)],
                ));
            }
        }
    }

    /// The `key.volt.replay` mapping (R by default): the button's tap.
    pub(super) fn replay_shortcut(&mut self, key: KeyCode, repeat: bool) -> bool {
        if repeat {
            return false;
        }
        if horizon_client::keybinds::key_name(key)
            != Some(self.settings.keys.get("key.volt.replay"))
        {
            return false;
        }
        self.replay_action();
        true
    }

    /// One world tick's camera sample while recording; paused holds the take.
    pub(super) fn tick_replay(&mut self) {
        if !self.settings.extras.volt_qol.replay_buffer
            || self.replay_state != ReplayState::Recording
        {
            return;
        }
        let Some(eye) = self.eye_position() else {
            return;
        };
        if let Some(track) = &mut self.replay_track {
            track.record(self.world.game_time, eye, self.yaw, self.pitch);
        }
    }

    /// The replay button's rect in HUD units, from its config (`None` while
    /// the row is off or a menu is open).
    pub(super) fn replay_button_rect(&self) -> Option<[f32; 4]> {
        if self.screen != super::Screen::Playing {
            return None;
        }
        let rq = &self.settings.extras.volt_qol;
        if !rq.replay_button {
            return None;
        }
        let batch = HudBatch::new(self.touch_size());
        let s = f32::from(rq.replay_btn_size);
        let x = (batch.width - s) * f32::from(rq.replay_btn_x_pct) / 100.0;
        let y = (batch.height - s) * f32::from(rq.replay_btn_y_pct) / 100.0;
        Some([x, y, s, s])
    }

    /// Whether a touch (window pixels) landed on the replay button.
    pub(super) fn replay_button_at(&self, p: [f32; 2]) -> bool {
        let Some([x, y, w, h]) = self.replay_button_rect() else {
            return false;
        };
        let batch = HudBatch::new(self.touch_size());
        let (px, py) = (p[0] / batch.scale, p[1] / batch.scale);
        px >= x && px < x + w && py >= y && py < y + h
    }

    /// The held finger moved: decide the mode, then move the button, or with
    /// a long press held, drag up-down for its size and left-right for its
    /// opacity.
    pub(super) fn replay_touch_drag(&mut self, p: [f32; 2]) {
        let Some(mut st) = self.replay_touch.take() else {
            return;
        };
        let batch = HudBatch::new(self.touch_size());
        let (px, py) = (p[0] / batch.scale, p[1] / batch.scale);
        let moved = ((px - st.start[0] / batch.scale).powi(2)
            + (py - st.start[1] / batch.scale).powi(2))
        .sqrt();
        if st.mode == 0 {
            if moved >= DRAG_START {
                st.mode = 1;
            } else if st.began.elapsed() >= HOLD {
                st.mode = 2;
                st.mode_start = [px, py];
                st.size = self.settings.extras.volt_qol.replay_btn_size;
                st.opacity = self.settings.extras.volt_qol.replay_btn_opacity_pct;
            } else {
                self.replay_touch = Some(st);
                return;
            }
        }
        let rq = &mut self.settings.extras.volt_qol;
        if st.mode == 1 {
            // The button's centre follows the finger, in percent of the screen.
            let s = f32::from(rq.replay_btn_size);
            let cx = (px - s / 2.0) / (batch.width - s).max(1.0) * 100.0;
            let cy = (py - s / 2.0) / (batch.height - s).max(1.0) * 100.0;
            rq.replay_btn_x_pct = (cx.round() as u16).clamp(0, 100);
            rq.replay_btn_y_pct = (cy.round() as u16).clamp(0, 100);
        } else {
            // Up-down resizes, left-right fades, from the long press's start.
            let size = f32::from(st.size) + (py - st.mode_start[1]);
            let opacity = f32::from(st.opacity) + (px - st.mode_start[0]);
            rq.replay_btn_size = (size.round() as u16).clamp(22, 72);
            rq.replay_btn_opacity_pct = (opacity.round() as u16).clamp(25, 100);
        }
        self.replay_touch = Some(st);
    }

    /// The finger lifted: a tap acts (start, pause, end); a drag persists the
    /// button's spot to options.txt like the options screen would.
    pub(super) fn replay_touch_end(&mut self, p: [f32; 2]) {
        let Some(st) = self.replay_touch.take() else {
            return;
        };
        let batch = HudBatch::new(self.touch_size());
        let moved = ((p[0] / batch.scale - st.start[0] / batch.scale).powi(2)
            + (p[1] / batch.scale - st.start[1] / batch.scale).powi(2))
        .sqrt();
        if st.mode == 0 && moved < DRAG_START && st.began.elapsed() < HOLD {
            self.replay_action();
        } else {
            self.settings.save(&Self::options_path());
        }
    }
}

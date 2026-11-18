//! Rezyth Settings' replay recorder (`volt_qol.replay_buffer`): the floating
//! button's action (or the `key.volt.replay` key, or a touch button bound to
//! it) cycles idle → recording ⇄ paused → idle, sampling the camera once per
//! world tick while recording and writing the track as JSON to the player's
//! `replays/` folder on the way out. The launcher's Replays page plays the
//! takes back (timeline, speeds, the camera's path); in game, the
//! `key.volt.replay_watch` key (K) flies the newest take's recorded camera in
//! the world it was recorded in, first person.

use super::Game;
use horizon_client::volt_features::{ReplayState, ReplayTrack};
use horizon_client::{hud::HudBatch, logging, text::Text};
use std::path::PathBuf;
use std::time::Duration;
use winit::keyboard::KeyCode;

/// An in-game playback: the take, how far into it the camera is (in world
/// ticks), whether it runs, and at what speed.
pub(super) struct ReplayWatch {
    pub(super) name: String,
    track: ReplayTrack,
    playhead: f64,
    playing: bool,
    speed: f32,
}

impl ReplayWatch {
    /// The camera pose at the playhead, in the take's world ticks.
    pub(super) fn pose(&self) -> Option<([f64; 3], f32, f32)> {
        let first = self.track.samples.first()?.tick;
        self.track.pose_at(first + self.playhead as i64)
    }

    /// How far the take runs, in world ticks.
    fn length(&self) -> f64 {
        self.track.length_ticks() as f64
    }

    /// `m:ss / m:ss` for the HUD.
    pub(super) fn clock(&self) -> (String, String) {
        let at = (self.playhead / 20.0).max(0.0);
        let length = (self.length() / 20.0).max(0.0);
        (clock(at), clock(length))
    }
}

/// Seconds as `m:ss`.
fn clock(seconds: f64) -> String {
    let total = seconds.round() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

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
                // Watching gives way to a new recording.
                self.replay_watch = None;
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

    /// The `key.volt.replay_watch` mapping (K by default): the newest take
    /// starts playing in its own dimension, the key again stops it.
    pub(super) fn replay_watch_shortcut(&mut self, key: KeyCode, repeat: bool) -> bool {
        if repeat {
            return false;
        }
        if horizon_client::keybinds::key_name(key)
            != Some(self.settings.keys.get("key.volt.replay_watch"))
        {
            return false;
        }
        if self.replay_watch.take().is_some() {
            self.system_message(Text::translate("horizon.replay.watch.stopped", &[]));
        } else {
            self.start_replay_watch();
        }
        true
    }

    /// Opens the newest take in `replays/` and starts the camera on it.
    fn start_replay_watch(&mut self) {
        if self.replay_state != ReplayState::Idle {
            self.system_message(Text::translate("horizon.replay.watch.recording", &[]));
            return;
        }
        let Some((name, path)) = newest_take() else {
            self.system_message(Text::translate("horizon.replay.watch.missing", &[]));
            return;
        };
        let loaded = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|json| serde_json::from_str::<ReplayTrack>(&json).map_err(|e| e.to_string()));
        let track = match loaded {
            Ok(track) => track,
            Err(e) => {
                self.system_message(Text::translate(
                    "horizon.replay.watch.failed",
                    &[Text::literal(e)],
                ));
                return;
            }
        };
        if track.dimension != self.dim.id() {
            let short = track
                .dimension
                .rsplit(':')
                .next()
                .unwrap_or(&track.dimension);
            self.system_message(Text::translate(
                "horizon.replay.watch.dimension",
                &[Text::literal(short.to_owned())],
            ));
            return;
        }
        if track.samples.is_empty() {
            self.system_message(Text::translate("horizon.replay.watch.empty", &[]));
            return;
        }
        self.system_message(Text::translate(
            "horizon.replay.watch.started",
            &[Text::literal(name.clone())],
        ));
        self.replay_watch = Some(ReplayWatch {
            name,
            track,
            playhead: 0.0,
            playing: true,
            speed: 1.0,
        });
    }

    /// One world tick's camera sample while recording; paused holds the take.
    /// An open playback advances one world tick per tick at its speed.
    pub(super) fn tick_replay(&mut self) {
        if let Some(watch) = &mut self.replay_watch {
            if watch.playing {
                watch.playhead += f64::from(watch.speed);
                if watch.playhead >= watch.length() {
                    watch.playhead = watch.length();
                    watch.playing = false;
                }
            }
        }
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

/// The newest take in the player's `replays/` folder, by file name (the
/// recorder stamps it with the Unix time): its display name and path.
fn newest_take() -> Option<(String, PathBuf)> {
    let dir = logging::player_dir().join("replays");
    let mut takes: Vec<(String, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            (name.ends_with(".voltreplay.json") && path.is_file()).then_some((name, path))
        })
        .collect();
    takes.sort_by(|a, b| b.0.cmp(&a.0));
    takes.into_iter().next()
}

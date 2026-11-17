//! Rezyth Settings' replay recorder (`volt_qol.replay_buffer`): the floating
//! button's action (or the `key.volt.replay` key, or a touch button bound to
//! it) cycles idle → recording ⇄ paused → idle, sampling the camera once per
//! world tick while recording and writing the track as JSON to the player's
//! `replays/` folder on the way out. The launcher-side player (timeline,
//! speeds, free camera) reads these files and is not built yet; the takes are
//! real, complete camera tracks.

use super::Game;
use horizon_client::volt_features::{ReplayState, ReplayTrack};
use horizon_client::{logging, text::Text};
use winit::keyboard::KeyCode;

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
}

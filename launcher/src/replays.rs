//! An instance's replays: the camera-track takes Rezyth Settings' replay recorder saves into
//! `replays/*.voltreplay.json` in its game folder (the player's own folder, as `--player-dir`
//! moves it).
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// One saved take: its file and the header its JSON carries.
#[derive(Clone, Debug, PartialEq)]
pub struct Replay {
    pub path: PathBuf,
    /// Its file name.
    pub name: String,
    /// The dimension id the take started in (`minecraft:overworld`).
    pub dimension: String,
    /// The world's game time at the first sample.
    pub start_game_time: i64,
    /// The take's length in world ticks (first to last sample).
    pub length_ticks: i64,
    pub samples: usize,
    /// When the file was written.
    pub taken: SystemTime,
}

/// One camera sample, as the file stores it.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub struct Sample {
    pub tick: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

/// The camera track of a take, as its JSON stores it.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Track {
    pub dimension: String,
    pub start_game_time: i64,
    pub samples: Vec<Sample>,
}

/// The folder the recorder saves into for the game folder `home`.
pub fn folder(home: &Path) -> PathBuf {
    home.join("replays")
}

/// The takes in `home`'s replays folder, the newest first.
pub fn list(home: &Path) -> Vec<Replay> {
    let mut replays: Vec<Replay> = fs::read_dir(folder(home))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let is_take = path
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.eq_ignore_ascii_case("json"));
            let meta = e.metadata().ok().filter(|m| m.is_file() && is_take)?;
            let taken = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let track = load(&path).ok()?;
            Some(Replay {
                name: e.file_name().to_string_lossy().into_owned(),
                length_ticks: length_of(&track),
                samples: track.samples.len(),
                dimension: track.dimension,
                start_game_time: track.start_game_time,
                path,
                taken,
            })
        })
        .collect();
    replays.sort_by(|a, b| b.taken.cmp(&a.taken).then_with(|| b.name.cmp(&a.name)));
    replays
}

/// The take a file holds (`load` of [`list`]'s entries, for the viewer).
pub fn load(path: &Path) -> Result<Track, String> {
    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// The take's length in world ticks.
fn length_of(track: &Track) -> i64 {
    match (track.samples.first(), track.samples.last()) {
        (Some(first), Some(last)) => last.tick - first.tick,
        _ => 0,
    }
}

/// The camera pose at `tick`, interpolated between the samples either side of
/// it (`None` for an empty take or a tick outside it).
pub fn pose_at(track: &Track, tick: i64) -> Option<(f64, f64, f64, f32, f32)> {
    let samples = &track.samples;
    let last = samples.len().checked_sub(1)?;
    let first_tick = samples[0].tick;
    if tick <= first_tick {
        let s = &samples[0];
        return Some((s.x, s.y, s.z, s.yaw, s.pitch));
    }
    let last_tick = samples[last].tick;
    if tick >= last_tick {
        let s = &samples[last];
        return Some((s.x, s.y, s.z, s.yaw, s.pitch));
    }
    // The samples are one per world tick: the slot is direct, one step back
    // while the tick is not exact.
    let i = (tick - first_tick) as usize;
    let (a, b) = (samples[i.min(last)], samples[i.max(1).min(last)]);
    let (a, b) = if a.tick <= tick && b.tick >= tick {
        (a, b)
    } else {
        let mut lo = 0usize;
        let mut hi = last;
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if samples[mid].tick <= tick {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (samples[lo], samples[hi])
    };
    let span = (b.tick - a.tick).max(1) as f64;
    let t = (tick - a.tick) as f64 / span;
    let lerp = |a: f64, b: f64| a + (b - a) * t;
    Some((
        lerp(a.x, b.x),
        lerp(a.y, b.y),
        lerp(a.z, b.z),
        a.yaw + (b.yaw - a.yaw) * t as f32,
        a.pitch + (b.pitch - a.pitch) * t as f32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(samples: Vec<(i64, f64)>) -> Track {
        Track {
            dimension: "minecraft:overworld".into(),
            start_game_time: 0,
            samples: samples
                .into_iter()
                .map(|(tick, x)| Sample {
                    tick,
                    x,
                    y: 64.0,
                    z: 0.0,
                    yaw: 0.0,
                    pitch: 0.0,
                })
                .collect(),
        }
    }

    /// The pose at a tick interpolates between the samples either side.
    #[test]
    fn pose_at_interpolates() {
        let t = track(vec![(100, 0.0), (110, 10.0), (120, 20.0)]);
        let (x, _, _, _, _) = pose_at(&t, 105).unwrap();
        assert!((x - 5.0).abs() < 1e-9);
        let (x, _, _, _, _) = pose_at(&t, 100).unwrap();
        assert!((x - 0.0).abs() < 1e-9);
        let (x, _, _, _, _) = pose_at(&t, 130).unwrap();
        assert!((x - 20.0).abs() < 1e-9, "past the end clamps to the last");
        assert!(pose_at(&track(vec![]), 100).is_none());
    }
}

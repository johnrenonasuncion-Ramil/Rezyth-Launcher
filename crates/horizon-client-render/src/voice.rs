//! Rezyth Settings' 3D proximity voice (`volt_chat.voice_*`): the microphone
//! capture chain, the loopback test path and the network transport. The
//! capture runs on the audio input stream (the `audio` feature; without it the
//! buffer stays empty and the HUD shows the mic as unavailable), the output
//! mix drains it while the loopback test path is on. The transport carries the
//! same gained, gated samples over the session connection as `horizon:voice`
//! custom payloads; a Horizon server relays each chunk to the other players in
//! the dimension, with the sender's server position, and their clients mix it
//! at the proximity gain below. A vanilla server ignores the unknown channel,
//! so voice stays between Horizon players.

/// The RMS level (0..1) of a capture chunk.
pub fn voice_rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let sum = frame.iter().map(|x| x * x).sum::<f32>();
    (sum / frame.len() as f32).sqrt()
}

/// The voice-activity gate: true while `rms` reaches `vad_threshold_pct` % of
/// full scale (the row's 0..=100 %).
pub fn voice_gate(rms: f32, vad_threshold_pct: u16) -> bool {
    rms * 100.0 >= f32::from(vad_threshold_pct)
}

/// The mic gain on one captured sample (`voice_mic_gain_pct`), clamped to the
/// full scale so a hot mic doesn't wrap.
pub fn voice_gain(x: f32, mic_gain_pct: u16) -> f32 {
    (x * f32::from(mic_gain_pct) / 100.0).clamp(-1.0, 1.0)
}

/// The output gain on one playback sample (`voice_output_gain_pct`). Unclamped:
/// the mixer's limiter takes anything past the full scale.
pub fn voice_out_gain(x: f32, out_gain_pct: u16) -> f32 {
    x * f32::from(out_gain_pct) / 100.0
}

/// The proximity falloff for a voice `at` blocks from the listener
/// (`voice_distance_blocks`): linear from full at the listener to silent at
/// the reach. Your own loopback (at 0) always plays at full.
pub fn voice_distance_gain(distance_blocks: u16, at: f32) -> f32 {
    let reach = f32::from(distance_blocks).max(1.0);
    (1.0 - at / reach).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The RMS of a known chunk: silence, a full-scale sine-ish burst.
    #[test]
    fn voice_rms_measures_the_level() {
        assert_eq!(voice_rms(&[]), 0.0);
        assert_eq!(voice_rms(&[0.0; 16]), 0.0);
        assert_eq!(voice_rms(&[0.5; 16]), 0.5);
        let mixed = voice_rms(&[0.5, -0.5, 0.5, -0.5]);
        assert!((mixed - 0.5).abs() < 1e-6);
    }

    /// The activity gate at the row's threshold percentage.
    #[test]
    fn voice_gate_at_the_threshold() {
        assert!(!voice_gate(0.0, 22));
        assert!(!voice_gate(0.219, 22));
        assert!(voice_gate(0.221, 22));
        assert!(voice_gate(0.0, 0), "a 0% gate always passes");
        assert!(!voice_gate(1.0, 101));
    }

    /// The mic gain scales and clamps to the full scale.
    #[test]
    fn voice_gain_scales_and_clamps() {
        assert_eq!(voice_gain(0.25, 100), 0.25);
        assert_eq!(voice_gain(0.25, 200), 0.5);
        assert_eq!(voice_gain(0.9, 200), 1.0, "clamped to the full scale");
        assert_eq!(voice_gain(-0.9, 200), -1.0);
        assert_eq!(voice_gain(0.5, 50), 0.25);
    }

    /// The output gain scales without clamping (the mixer limits).
    #[test]
    fn voice_out_gain_scales() {
        assert_eq!(voice_out_gain(0.25, 100), 0.25);
        assert_eq!(voice_out_gain(0.25, 200), 0.5);
        assert_eq!(voice_out_gain(0.25, 0), 0.0);
    }

    /// The proximity falloff: full at the listener, silent at the reach.
    #[test]
    fn voice_distance_gain_falls_off() {
        assert_eq!(voice_distance_gain(48, 0.0), 1.0);
        assert_eq!(voice_distance_gain(48, 24.0), 0.5);
        assert_eq!(voice_distance_gain(48, 96.0), 0.0);
        assert_eq!(
            voice_distance_gain(0, 0.0),
            1.0,
            "a zero reach stays audible"
        );
    }
}

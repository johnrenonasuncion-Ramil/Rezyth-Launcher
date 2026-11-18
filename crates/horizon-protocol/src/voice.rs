//! Rezyth Settings' proximity voice transport (`horizon:voice`): the capture
//! chunks a player's client sends over its session connection, relayed by the
//! hub to everyone else. The samples are mono f32 at the capture rate the
//! frame carries; a client that receives one resamples to its own output.
//!
//! A guest sends a [`frame`]; the hub forwards a [`relay`] with the sender's
//! uuid and server-known position in front, so receivers place the voice
//! without trusting anything the sender wrote about itself.

/// The custom-payload channel the frames travel on.
pub const CHANNEL: &str = "horizon:voice";

/// A capture chunk a client sends: samples at most, so a stuck sender cannot
/// flood a connection.
pub const MAX_SAMPLES: usize = 8192;

/// One chunk's payload as the sender's client sends it.
pub fn frame(rate: u32, samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(6 + samples.len() * 4);
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(samples.len().min(MAX_SAMPLES) as u16).to_le_bytes());
    for s in samples.iter().take(MAX_SAMPLES) {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// The same chunk as the hub relays it: the sender's uuid and position in
/// front, so the receivers know whose voice and where it comes from.
pub fn relay(sender: u128, at: [f32; 3], rate: u32, samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(28 + 6 + samples.len() * 4);
    out.extend_from_slice(&sender.to_le_bytes());
    for v in at {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&frame(rate, samples));
    out
}

/// A parsed payload: whose voice (`sender` and `at`, on a relayed payload
/// only), the capture rate and the samples.
pub struct Incoming {
    pub sender: Option<u128>,
    pub at: [f32; 3],
    pub rate: u32,
    pub samples: Vec<f32>,
}

/// Parses a payload a guest sent the hub (no sender: the hub knows it).
pub fn parse_guest(data: &[u8]) -> Option<Incoming> {
    parse(data, false)
}

/// Parses a payload the hub relayed (the sender's uuid and position in front).
pub fn parse_relay(data: &[u8]) -> Option<Incoming> {
    parse(data, true)
}

fn parse(data: &[u8], relayed: bool) -> Option<Incoming> {
    let head = if relayed { 28 } else { 0 };
    if data.len() < head + 6 {
        return None;
    }
    let mut at = [0.0f32; 3];
    let sender = relayed.then(|| {
        let uuid = u128::from_le_bytes(data[..16].try_into().expect("checked above"));
        for (i, v) in at.iter_mut().enumerate() {
            let b = &data[16 + i * 4..20 + i * 4];
            *v = f32::from_le_bytes(b.try_into().expect("checked above"));
        }
        uuid
    });
    let rate = u32::from_le_bytes(data[head..head + 4].try_into().expect("checked above"));
    let count =
        u16::from_le_bytes(data[head + 4..head + 6].try_into().expect("checked above")) as usize;
    let mut cursor = head + 6;
    if rate == 0 || count > MAX_SAMPLES || data.len() < cursor + count * 4 {
        return None;
    }
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let b = &data[cursor..cursor + 4];
        samples.push(f32::from_le_bytes(b.try_into().expect("checked above")));
        cursor += 4;
    }
    Some(Incoming {
        sender,
        at,
        rate,
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame round-trips; a relay keeps the sender and its position;
    /// garbage is refused.
    #[test]
    fn frames_round_trip_and_refuse_garbage() {
        let samples = vec![0.25_f32, -0.5, 1.0];
        let f = frame(48_000, &samples);
        let got = parse_guest(&f).unwrap();
        assert_eq!(got.sender, None);
        assert_eq!(got.rate, 48_000);
        assert_eq!(got.samples, samples);
        let r = relay(7, [1.5, 64.0, -3.25], 24_000, &samples);
        let got = parse_relay(&r).unwrap();
        assert_eq!(got.sender, Some(7));
        assert_eq!(got.at, [1.5, 64.0, -3.25]);
        assert_eq!(got.rate, 24_000);
        assert_eq!(got.samples, samples);
        assert!(parse_guest(&f[..3]).is_none());
        assert!(parse_relay(&r[..10]).is_none());
        assert!(
            parse_guest(&frame(0, &samples)).is_none(),
            "no rate, no frame"
        );
        // A sender cannot ask for more than the cap: the count is refused.
        let mut over = 48_000u32.to_le_bytes().to_vec();
        over.extend_from_slice(&(MAX_SAMPLES as u16 + 1).to_le_bytes());
        assert!(parse_guest(&over).is_none());
        assert_eq!(
            parse_guest(&frame(48_000, &vec![0.0; MAX_SAMPLES + 8]))
                .unwrap()
                .samples
                .len(),
            MAX_SAMPLES,
            "a chunk past the cap is cut to it"
        );
    }
}

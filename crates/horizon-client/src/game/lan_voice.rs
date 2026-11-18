//! Rezyth Settings' proximity voice on the server side (`horizon:voice`): a
//! Horizon client's capture chunk is checked and relayed to every other
//! Horizon client in the same dimension, with the sender's uuid and the
//! server's position for them in front. The server never stores the audio and
//! never plays it; the clients apply the proximity gain at their own listener.
use super::{Game, lan::LanHost};
use horizon_protocol::{
    packets::{configuration::CustomPayload, play::Clientbound as C},
    voice,
};

/// The relayed chunks one connection may send per server tick: about a
/// second of 48 kHz audio in chunks of a few thousand samples.
const MAX_CHUNKS_PER_TICK: u8 = 24;

impl Game<'_> {
    /// A `horizon:voice` payload from `conn`: relayed to the other Horizon
    /// players in the same dimension when it parses, at most
    /// [`MAX_CHUNKS_PER_TICK`] a tick (a flood is dropped, not disconnected:
    /// audio is best-effort).
    pub(super) fn guest_voice(&mut self, host: &LanHost, conn: u32, data: &[u8]) {
        let Some(guest) = host.hub.guest(conn) else {
            return;
        };
        if guest.brand != "horizon" || !guest.placed {
            return;
        }
        let Some(incoming) = voice::parse_guest(data) else {
            return;
        };
        if incoming.samples.is_empty() {
            return;
        }
        let sent = self.voice_relays.entry(conn).or_default();
        if *sent >= MAX_CHUNKS_PER_TICK {
            return;
        }
        *sent += 1;
        let at = [
            guest.position[0] as f32,
            guest.position[1] as f32,
            guest.position[2] as f32,
        ];
        let packet = C::CustomPayload(CustomPayload {
            channel: voice::CHANNEL.into(),
            data: voice::relay(guest.uuid, at, incoming.rate, &incoming.samples),
        });
        let dimension = guest.dimension.clone();
        for other in &host.hub.guests {
            if other.conn == conn || other.brand != "horizon" || !other.placed {
                continue;
            }
            if other.dimension == dimension {
                other.send(&packet);
            }
        }
    }
}

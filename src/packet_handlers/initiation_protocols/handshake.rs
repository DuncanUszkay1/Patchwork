use super::packet::Packet;
use super::TranslationUpdates;

// Called upon handshake
pub fn handle_handshake_packet(p: Packet) -> TranslationUpdates {
    TranslationUpdates::State(match p.clone() {
        Packet::Handshake(handshake) => handshake.next_state,
        _ => panic!("Invalid packet {:?}", p),
    })
}

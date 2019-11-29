use super::game_state::patchwork::{NewMapMessage, PatchworkStateOperations, Peer};

use std::sync::mpsc::Sender;

// this file probably doesn't need to exist at all
pub fn send_p2p_handshake(
    peer_address: String,
    peer_port: u16,
    patchwork_state: Sender<PatchworkStateOperations>,
) {
    patchwork_state
        .send(PatchworkStateOperations::New(NewMapMessage {
            peer: Peer {
                port: peer_port,
                address: peer_address,
            },
        }))
        .unwrap();
}

use super::game_state::patchwork::{NewMapMessage, PatchworkStateOperations, Peer};
use super::messenger::{MessengerOperations, NewConnectionMessage, SendPacketMessage};
use super::packet::{Handshake, Packet};
use super::server;

use std::io;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub fn send_p2p_handshake(
    peer_address: String,
    peer_port: u16,
    messenger: Sender<MessengerOperations>,
    patchwork_state: Sender<PatchworkStateOperations>,
) {
    patchwork_state.send(PatchworkStateOperations::New(NewMapMessage {
        peer: Peer {
            port: peer_port,
            address: peer_address,
        },
    }));
}

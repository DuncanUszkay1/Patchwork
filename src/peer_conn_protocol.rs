use super::messenger::{MessengerOperations, NewConnectionMessage, SendPacketMessage};
use super::packet::{Handshake, Packet};
use super::server;

use std::sync::mpsc::Sender;
use uuid::Uuid;

pub fn send_p2p_handshake(
    conn_id: u64,
    peer_address: String,
    peer_port: u16,
    messenger: Sender<MessengerOperations>,
) -> Uuid {
    let stream = server::new_connection(peer_address.clone(), peer_port);
    messenger
        .send(MessengerOperations::New(NewConnectionMessage {
            conn_id,
            socket: stream.try_clone().unwrap(),
        }))
        .unwrap();

    send_packet!(
        messenger,
        conn_id,
        Packet::Handshake(Handshake {
            protocol_version: 404,
            server_address: peer_address.clone(),
            server_port: peer_port,
            next_state: 6,
        })
    )
    .unwrap();

    Uuid::new_v4()
}

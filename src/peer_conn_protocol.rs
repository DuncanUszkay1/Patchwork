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
) -> Result<Uuid, io::Error> {
    let conn_id = Uuid::new_v4();
    let stream = server::new_connection(peer_address.clone(), peer_port)?;
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

    Ok(Uuid::new_v4())
}
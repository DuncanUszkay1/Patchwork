use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet::{Handshake, Packet};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

pub enum P2POperations {
    New(NewPeerConnection),
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub conn_id: u64,
    pub peer_address: String,
    pub peer_port: u16,
}

#[derive(Debug, Clone)]
pub struct NewPeerConnection {
    pub conn_id: u64,
    pub peer: Peer,
}

pub fn send_p2p_handshake(
    conn_id: u64,
    peer_address: String,
    peer_port: u16,
    messenger: Sender<MessengerOperations>,
    receiver: Receiver<P2POperations>,
) {
    while let Ok(msg) = receiver.recv() {
        match msg {
            P2POperations::New(_msg) => {
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
            }
        }
    }
}

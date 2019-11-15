use super::messenger::{MessengerOperations, SendPacketMessage, NewConnectionMessage};
use super::packet::{Packet, Handshake};
use std::net::TcpStream;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

pub enum P2POperation {
    New(NewPeerConnection),
    Send(P2PHandshake),
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub conn_id: u64,
    pub peer_address: String,
    pub peer_port: u16,
    pub subscriber: bool,
}

#[derive(Debug, Clone)]
pub struct NewPeerConnection {
    pub conn_id: u64,
    pub peer: Peer,
}

#[derive(Debug, Clone)]
pub struct P2PHandshake {
    pub conn_id: u64,
    pub peer: Peer,
}

pub fn start_p2p_state(
    receiver: Receiver<P2POperation>,
    messenger: Sender<MessengerOperations>, 
) {
    let mut peers = HashMap::<u64, Peer>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            P2POperation::New(msg) => {
                peers.insert(msg.conn_id, msg.peer);
            },
            P2POperation::Send(msg) => {
                peers.values().for_each(|peer| {
                    let peer_clone = peer.clone();
                    let peer_info = format!("{}{}", peer_clone.peer_address, peer_clone.peer_port.to_string());
                    let mut stream = TcpStream::connect(peer_info).unwrap();
                    messenger.send(MessengerOperations::New(NewConnectionMessage {
                        conn_id: msg.conn_id,
                        socket: stream.try_clone().unwrap(),
                    }));

                    send_packet!(
                        messenger,
                        msg.conn_id,
                        Packet::Handshake(Handshake {
                            protocol_version: 404,
                            server_address: peer_clone.peer_address,
                            server_port: peer_clone.peer_port,
                            next_state: 6,
                        })
                    )
                    .unwrap();
                    
                })
            }
        }
    }
}


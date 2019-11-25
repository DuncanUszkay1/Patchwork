use super::messenger::{MessengerOperations, NewConnectionMessage, SendPacketMessage};
use super::packet::{Handshake, Packet};
use super::server;

use std::io;
use std::sync::mpsc::Sender;
use uuid::Uuid;
use std::collections::HashMap;
use std::net::TcpStream;

pub struct Peer {
    pub peer_address: String,
    pub peer_port: u16,
    pub stream: TcpStream,
}

impl Peer {
    pub fn send_p2p_handshake(
        &mut self,
        conn_id: u64,
        messenger: Sender<MessengerOperations>,
    ) -> Result<Uuid, io::Error> {
        
        messenger
            .send(MessengerOperations::New(NewConnectionMessage {
                conn_id,
                socket: self.stream.try_clone().unwrap(),
            }))
            .unwrap();
    
        send_packet!(
            messenger,
            conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: self.peer_address.clone(),
                server_port: self.peer_port,
                next_state: 6,
            })
        )
        .unwrap();
    
        Ok(Uuid::new_v4())
    }
}

pub struct PeerManager {
    pub peer_map: HashMap::<u64, Peer>,
}

impl PeerManager{
    pub fn new() -> PeerManager{
        let peer_map = HashMap::<u64, Peer>::new();
        PeerManager{peer_map}
    }

    pub fn connect_to_peer(
        &mut self,
        conn_id: u64,
        peer_address: String,
        peer_port: u16,
        messenger: Sender<MessengerOperations>,
    ) -> Result<Uuid, io::Error> {
        let stream = server::new_connection(peer_address.clone(), peer_port)?;
        let mut peer = Peer{peer_address, peer_port, stream};

        match(peer.send_p2p_handshake(conn_id, messenger)){
            Uuid => {
                self.peer_map.insert(conn_id, peer);
                println!("Peer connection succeded");
            },
            _ => {
                println!("New peer connection failed");
            }
        }
        Ok(Uuid::new_v4())
            
    }
}
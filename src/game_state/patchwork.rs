use super::messenger::{NewConnectionMessage, BroadcastPacketMessage, MessengerOperations, SendPacketMessage};
use super::server;
use super::packet::{Packet, Handshake};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub enum PatchworkStateOperations {
    New(NewMapMessage),
    Report(ReportMessage),
}

#[derive(Debug)]
pub struct NewMapMessage {
    pub peer: Peer,
}

#[derive(Debug)]
pub struct ReportMessage {
    pub conn_id: Uuid,
}

#[derive(Debug, Clone)]
struct Patchwork {
    pub maps: Vec<Map>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub port: u16,
    pub address: String,
}

impl Patchwork {
    pub fn new() -> Patchwork {
        Patchwork {
            maps: Vec::<Map>::new(),
        }
    }

    pub fn add_map(&mut self, peer: Peer) -> Map {
        let map = Map {
            position: self.next_position(),
            peer,
            entity_id_block: self.next_entity_id_block(),
        };
        self.maps.push(map.clone());
        println!("patchwork maps {:?}", self.maps);
        map
    }

    // get the next block of size 1000 entity ids assigned to this map
    fn next_entity_id_block(&self) -> i32 {
        (self.maps.len() + 1) as i32
    }

    // For now, just line up all the maps in a row
    fn next_position(&self) -> Position {
        let len = self.maps.len() as i32;
        Position { x: len + 1, z: 0 }
    }
}

#[derive(Debug, Clone)]
struct Map {
    pub position: Position,
    pub entity_id_block: i32,
    pub peer: Peer,
}

impl Map {
    fn connect(self, messenger: Sender<MessengerOperations>) {
        let conn_id = Uuid::new_v4();
        if let Ok(stream) = server::new_connection(self.peer.address.clone(), self.peer.port) {
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
                    server_address: self.peer.address.clone(),
                    server_port: self.peer.port,
                    next_state: 6,
                })
            )
            .unwrap();

            send_packet!(
                messenger,
                conn_id,
                Packet::Handshake(Handshake {
                    protocol_version: 404,
                    server_address: self.peer.address.clone(),
                    server_port: self.peer.port,
                    next_state: 6,
                })
            )
            .unwrap();
        };
    }
}

//This probably belongs at an entity level, but since we don't have a real concept of entities yet
//this'll do
//Ignoring angle since we haven't implemented that datatype just yet
#[derive(Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub z: i32,
}

pub fn start_patchwork_state(
    receiver: Receiver<PatchworkStateOperations>,
    messenger: Sender<MessengerOperations>,
) {
    let mut patchwork = Patchwork::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PatchworkStateOperations::New(msg) => {
                patchwork.add_map(msg.peer).connect(messenger.clone());
            }
            PatchworkStateOperations::Report(msg) => {
                unimplemented!("Don't know how to report this yet");
            }
        }
    }
}

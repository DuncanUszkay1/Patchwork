use super::messenger::{MessengerOperations, NewConnectionMessage, SendPacketMessage};
use super::packet::{Handshake, Packet};
use super::packet_processor::{
    PacketProcessorOperations, TranslationDataMessage, TranslationUpdates,
};
use super::server;
use std::io;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use uuid::Uuid;

pub const ENTITY_ID_BLOCK_SIZE: i32 = 1000;
pub const CHUNK_SIZE: i32 = 16;

pub enum PatchworkStateOperations {
    New(NewMapMessage),
    Report,
}

#[derive(Debug)]
pub struct NewMapMessage {
    pub peer: Peer,
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

pub fn start(
    receiver: Receiver<PatchworkStateOperations>,
    messenger: Sender<MessengerOperations>,
    inbound_packet_processor: Sender<PacketProcessorOperations>,
) {
    let mut patchwork = Patchwork::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PatchworkStateOperations::New(msg) => patchwork.add_peer_map(
                msg.peer,
                messenger.clone(),
                inbound_packet_processor.clone(),
            ),
            PatchworkStateOperations::Report => {
                patchwork.clone().report(messenger.clone());
            }
        }
    }
}

impl Patchwork {
    pub fn new() -> Patchwork {
        let mut patchwork = Patchwork { maps: Vec::new() };
        patchwork.create_local_map();
        patchwork
    }

    pub fn create_local_map(&mut self) {
        self.maps.push(Map::Local(LocalMap {
            position: self.next_position(),
            entity_id_block: self.next_entity_id_block(),
        }));
    }

    pub fn add_peer_map(
        &mut self,
        peer: Peer,
        messenger: Sender<MessengerOperations>,
        inbound_packet_processor: Sender<PacketProcessorOperations>,
    ) {
        if let Ok(map) = RemoteMap::try_new(
            messenger,
            inbound_packet_processor,
            peer,
            self.next_position(),
            self.next_entity_id_block(),
        ) {
            self.maps.push(Map::Remote(map));
        }
    }

    pub fn report(self, messenger: Sender<MessengerOperations>) {
        self.maps
            .into_iter()
            .for_each(|map| map.report(messenger.clone()));
    }

    // get the next block of size 1000 entity ids assigned to this map
    fn next_entity_id_block(&self) -> i32 {
        (self.maps.len() + 1) as i32
    }

    // For now, just line up all the maps in a row
    fn next_position(&self) -> Position {
        let len = self.maps.len() as i32;
        Position { x: len, z: 0 }
    }
}

trait Reportable {
    fn report(&self, messenger: Sender<MessengerOperations>);
}

#[derive(Debug, Clone)]
enum Map {
    Local(LocalMap),
    Remote(RemoteMap),
}

#[derive(Debug, Clone)]
struct RemoteMap {
    pub position: Position,
    pub entity_id_block: i32,
    pub conn_id: Uuid,
}

#[derive(Debug, Clone)]
struct LocalMap {
    pub position: Position,
    pub entity_id_block: i32,
    // pub block_state_sender
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub z: i32,
}

impl Reportable for Map {
    fn report(&self, messenger: Sender<MessengerOperations>) {
        match self {
            Map::Remote(map) => {
                map.report(messenger);
            }
            Map::Local(map) => {
                map.report(messenger);
            }
        }
    }
}

impl Reportable for RemoteMap {
    fn report(&self, messenger: Sender<MessengerOperations>) {
        send_packet!(
            messenger,
            self.conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: String::from(""), //Neither of these fields are actually used
                server_port: 0,
                next_state: 5,
            })
        )
        .unwrap();
    }
}

impl Reportable for LocalMap {
    fn report(&self, _messenger: Sender<MessengerOperations>) {
        // Eventaully this method will call the block state reporter, which will belong to the
        // local map. For now this does nothing
    }
}

impl RemoteMap {
    pub fn try_new(
        messenger: Sender<MessengerOperations>,
        inbound_packet_processor: Sender<PacketProcessorOperations>,
        peer: Peer,
        position: Position,
        entity_id_block: i32,
    ) -> Result<RemoteMap, io::Error> {
        let conn_id = Uuid::new_v4();
        let stream = server::new_connection(peer.address.clone(), peer.port)?;
        messenger
            .send(MessengerOperations::New(NewConnectionMessage {
                conn_id,
                socket: stream.try_clone().unwrap(),
            }))
            .unwrap();
        inbound_packet_processor
            .send(PacketProcessorOperations::SetTranslationData(
                TranslationDataMessage {
                    conn_id,
                    updates: vec![
                        TranslationUpdates::State(5),
                        TranslationUpdates::EntityIdBlock(entity_id_block),
                        TranslationUpdates::XOrigin(position.x),
                    ],
                },
            ))
            .unwrap();

        let messenger_clone = messenger.clone();
        let inbound_packet_processor_clone = inbound_packet_processor.clone();
        thread::spawn(move || {
            server::handle_connection(
                stream.try_clone().unwrap(),
                inbound_packet_processor_clone,
                messenger_clone,
                conn_id,
            );
        });
        let map = RemoteMap {
            position,
            entity_id_block,
            conn_id,
        };
        send_packet!(
            messenger,
            conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: peer.address.clone(),
                server_port: peer.port,
                next_state: 6,
            })
        )
        .unwrap();

        //we send two packets because our protocol requires at least two packets to be sent
        //before it can do anything- the first is a handshake, then the second one it can
        //actually response to (it ignores the type of packet, so we just send it random data)
        //to be changed later with a real request packet
        send_packet!(
            messenger,
            conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: peer.address.clone(),
                server_port: peer.port,
                next_state: 6,
            })
        )
        .unwrap();
        Ok(map)
    }
}

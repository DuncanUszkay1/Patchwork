use super::map::{LocalMap, Map, Peer, Position, RemoteMap};
use super::messenger::MessengerOperations;
use super::packet::Packet;
use super::packet_processor::PacketProcessorOperations;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub const ENTITY_ID_BLOCK_SIZE: i32 = 1000;
pub const CHUNK_SIZE: i32 = 16;

pub enum PatchworkStateOperations {
    New(NewMapMessage),
    RoutePlayerPacket(RouteMessage),
    Report,
}

#[derive(Debug)]
pub struct NewMapMessage {
    pub peer: Peer,
}

#[derive(Debug)]
pub struct RouteMessage {
    pub packet: Packet,
    pub conn_id: Uuid,
}

#[derive(Debug, Clone)]
struct Patchwork {
    pub maps: Vec<Map>,
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
            PatchworkStateOperations::RoutePlayerPacket(msg) => {
                if let Some(position) = extract_map_position(msg.packet) {
                    let map_index = patchwork.clone().position_map_index(position);
                    println!("player on map {:?}", map_index);
                }
            }
            PatchworkStateOperations::Report => {
                patchwork.clone().report(messenger.clone());
            }
        }
    }
}

pub fn extract_map_position(packet: Packet) -> Option<Position> {
    match packet {
        Packet::PlayerPosition(packet) => Some(Position {
            x: (packet.x / 16.0) as i32,
            z: (packet.z / 16.0) as i32,
        }),
        Packet::PlayerPositionAndLook(packet) => Some(Position {
            x: (packet.x / 16.0) as i32,
            z: (packet.z / 16.0) as i32,
        }),
        _ => None,
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

    pub fn position_map_index(self, position: Position) -> usize {
        self.maps
            .into_iter()
            .position(|map| map.position() == position)
            .unwrap()
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
        self.maps.len() as i32
    }

    // For now, just line up all the maps in a row
    fn next_position(&self) -> Position {
        let len = self.maps.len() as i32;
        Position { x: len, z: 0 }
    }
}

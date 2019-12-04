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
            conn_id: Uuid::new_v4(),
        };
        self.maps.push(map.clone());
        map
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
        Position { x: len + 1, z: 0 }
    }
}

#[derive(Debug, Clone)]
struct Map {
    pub position: Position,
    pub entity_id_block: i32,
    pub peer: Peer,
    pub conn_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub z: i32,
}

impl Map {
    fn connect(
        &self,
        messenger: Sender<MessengerOperations>,
        inbound_packet_processor: Sender<PacketProcessorOperations>,
    ) -> Result<(), io::Error> {
        let stream = server::new_connection(self.peer.address.clone(), self.peer.port)?;
        messenger
            .send(MessengerOperations::New(NewConnectionMessage {
                conn_id: self.conn_id,
                socket: stream.try_clone().unwrap(),
            }))
            .unwrap();
        inbound_packet_processor
            .send(PacketProcessorOperations::SetTranslationData(
                TranslationDataMessage {
                    conn_id: self.conn_id,
                    update: TranslationUpdates::State(5),
                },
            ))
            .unwrap();
        inbound_packet_processor
            .send(PacketProcessorOperations::SetTranslationData(
                TranslationDataMessage {
                    conn_id: self.conn_id,
                    update: TranslationUpdates::EntityIdBlock(self.entity_id_block),
                },
            ))
            .unwrap();
        inbound_packet_processor
            .send(PacketProcessorOperations::SetTranslationData(
                TranslationDataMessage {
                    conn_id: self.conn_id,
                    update: TranslationUpdates::XOrigin(self.position.x),
                },
            ))
            .unwrap();
        let messenger_clone = messenger.clone();
        let inbound_packet_processor_clone = inbound_packet_processor.clone();
        let conn_id_clone = self.conn_id;
        thread::spawn(move || {
            server::handle_connection(
                stream.try_clone().unwrap(),
                inbound_packet_processor_clone,
                messenger_clone,
                conn_id_clone,
            );
        });
        Ok(())
    }

    fn create_event_listener(
        self,
        messenger: Sender<MessengerOperations>,
        inbound_packet_processor: Sender<PacketProcessorOperations>,
    ) {
        if let Ok(()) = self.connect(messenger.clone(), inbound_packet_processor) {
            send_packet!(
                messenger,
                self.conn_id,
                Packet::Handshake(Handshake {
                    protocol_version: 404,
                    server_address: self.peer.address.clone(),
                    server_port: self.peer.port,
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
                self.conn_id,
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

    fn report(self, messenger: Sender<MessengerOperations>) {
        send_packet!(
            messenger,
            self.conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: self.peer.address.clone(),
                server_port: self.peer.port,
                next_state: 5,
            })
        )
        .unwrap();
    }
}

pub fn start(
    receiver: Receiver<PatchworkStateOperations>,
    messenger: Sender<MessengerOperations>,
    inbound_packet_processor: Sender<PacketProcessorOperations>,
) {
    let mut patchwork = Patchwork::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PatchworkStateOperations::New(msg) => {
                patchwork
                    .add_map(msg.peer)
                    .create_event_listener(messenger.clone(), inbound_packet_processor.clone());
            }
            PatchworkStateOperations::Report => {
                patchwork.clone().report(messenger.clone());
            }
        }
    }
}

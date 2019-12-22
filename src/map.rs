use super::messenger::{MessengerOperations, NewConnectionMessage, SendPacketMessage};
use super::packet::{Handshake, Packet};
use super::packet_processor::{
    PacketProcessorOperations, TranslationDataMessage, TranslationUpdates,
};
use super::server;
use std::io;
use std::sync::mpsc::Sender;
use std::thread;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Map {
    Local(LocalMap),
    Remote(RemoteMap),
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub port: u16,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct RemoteMap {
    pub peer: Peer,
    pub position: Position,
    pub entity_id_block: i32,
    pub conn_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct LocalMap {
    pub position: Position,
    pub entity_id_block: i32,
    // pub block_state_sender
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: i32,
    pub z: i32,
}

impl Map {
    pub fn report(&self, messenger: Sender<MessengerOperations>) {
        match self {
            Map::Remote(map) => {
                send_packet!(
                    messenger,
                    map.conn_id,
                    Packet::Handshake(Handshake {
                        protocol_version: 404,
                        server_address: String::from(""), //Neither of these fields are actually used
                        server_port: 0,
                        next_state: 5,
                    })
                )
                .unwrap();
            }
            Map::Local(_map) => {}
        }
    }

    pub fn position(self) -> Position {
        match self {
            Map::Remote(map) => map.position,
            Map::Local(map) => map.position,
        }
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
            peer,
            position,
            entity_id_block,
            conn_id,
        };
        send_packet!(
            messenger,
            conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: String::from(""),
                server_port: 0,
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
                server_address: String::from(""),
                server_port: 0,
                next_state: 6,
            })
        )
        .unwrap();
        Ok(map)
    }
}

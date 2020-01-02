use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::PacketProcessor;
use super::packet::{Handshake, Packet};
use super::server;
use super::translation::TranslationUpdates;
use std::io;
use std::thread;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub peer: Peer,
    pub conn_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub port: u16,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub position: Position,
    pub entity_id_block: i32,
    pub peer_connection: Option<PeerConnection>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub z: i32,
}

impl Map {
    pub fn report<M: Messenger>(&self, messenger: M) {
        if let Some(peer_connection) = &self.peer_connection {
            messenger.send_packet(
                peer_connection.conn_id,
                Packet::Handshake(Handshake {
                    protocol_version: 404,
                    server_address: String::from(""), //Neither of these fields are actually used
                    server_port: 0,
                    next_state: 5,
                }),
            );
        }
    }

    pub fn new(position: Position, entity_id_block: i32) -> Map {
        Map {
            position,
            entity_id_block,
            peer_connection: None,
        }
    }

    pub fn connect<
        M: 'static + Messenger + Clone + Send,
        PP: 'static + PacketProcessor + Clone + Send,
    >(
        &self,
        messenger: M,
        inbound_packet_processor: PP,
        peer: Peer,
    ) -> Result<Map, io::Error> {
        let conn_id = Uuid::new_v4();
        let stream = server::new_connection(peer.address.clone(), peer.port)?;
        messenger.new_connection(conn_id, stream.try_clone().unwrap());
        inbound_packet_processor.set_translation_data(
            conn_id,
            vec![
                TranslationUpdates::State(5),
                TranslationUpdates::EntityIdBlock(self.entity_id_block),
                TranslationUpdates::XOrigin(self.position.x),
            ],
        );

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
        let map = Map {
            peer_connection: Some(PeerConnection { peer, conn_id }),
            position: self.position,
            entity_id_block: self.entity_id_block,
        };
        messenger.send_packet(
            conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: String::from(""),
                server_port: 0,
                next_state: 6,
            }),
        );

        //we send two packets because our protocol requires at least two packets to be sent
        //before it can do anything- the first is a handshake, then the second one it can
        //actually response to (it ignores the type of packet, so we just send it random data)
        //to be changed later with a real request packet
        messenger.send_packet(
            conn_id,
            Packet::Handshake(Handshake {
                protocol_version: 404,
                server_address: String::from(""),
                server_port: 0,
                next_state: 6,
            }),
        );
        Ok(map)
    }
}

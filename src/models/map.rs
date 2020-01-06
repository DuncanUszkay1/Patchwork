use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::PacketProcessor;
use super::interfaces::patchwork::PatchworkState;
use super::packet::{Handshake, Packet};
use super::server;
use super::translation::TranslationUpdates;

use std::net::TcpStream;
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
            trace!("Reporting map {:?}", self);
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
        PA: 'static + PatchworkState + Clone + Send,
    >(
        &self,
        messenger: M,
        inbound_packet_processor: PP,
        peer: Peer,
        patchwork_state: PA,
        map_index: usize,
    ) {
        let conn_id = Uuid::new_v4();
        let translation_updates = vec![
            TranslationUpdates::State(5),
            TranslationUpdates::EntityIdBlock(self.entity_id_block),
            TranslationUpdates::XOrigin(self.position.x),
        ];
        let peer_clone = peer.clone();
        let on_connection = move |stream: TcpStream| {
            messenger.new_connection(conn_id, stream.try_clone().unwrap());
            inbound_packet_processor.set_translation_data(conn_id, translation_updates);

            let messenger_clone = messenger.clone();
            let inbound_packet_processor_clone = inbound_packet_processor.clone();
            thread::spawn(move || {
                server::handle_connection(
                    stream.try_clone().unwrap(),
                    inbound_packet_processor_clone,
                    messenger_clone,
                    conn_id,
                    || {},
                );
            });
            messenger.send_packet(
                conn_id,
                Packet::Handshake(Handshake {
                    protocol_version: 404,
                    server_address: String::from(""),
                    server_port: 0,
                    next_state: 6,
                }),
            );
            patchwork_state.connect_map(
                map_index,
                PeerConnection {
                    peer: peer_clone,
                    conn_id,
                },
            );
        };
        thread::spawn(move || {
            server::wait_for_connection(peer.address.clone(), peer.port, on_connection);
        });
    }
}

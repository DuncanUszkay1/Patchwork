use super::packet::{translate_outgoing, write, Packet};
use super::packet_processor::{Map, TranslationInfo};
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use uuid::Uuid;

macro_rules! send_packet {
    ($messenger:expr, $conn_id:expr, $packet:expr) => {
        $messenger.send(MessengerOperations::Send(SendPacketMessage {
            conn_id: $conn_id,
            packet: $packet,
        }))
    };
}

macro_rules! broadcast_packet {
    ($messenger:expr, $packet:expr, $source_conn_id: expr, $local: expr) => {
        $messenger.send(MessengerOperations::Broadcast(BroadcastPacketMessage {
            packet: $packet,
            source_conn_id: $source_conn_id,
            local: $local,
        }))
    };
}

pub enum MessengerOperations {
    Send(SendPacketMessage),
    Broadcast(BroadcastPacketMessage),
    Subscribe(SubscribeMessage),
    New(NewConnectionMessage),
    UpdateTranslation(UpdateTranslationMessage),
}

#[derive(Debug)]
pub struct SendPacketMessage {
    pub conn_id: Uuid,
    pub packet: Packet,
}

#[derive(Debug)]
pub struct UpdateTranslationMessage {
    pub conn_id: Uuid,
    pub map: Map,
}

#[derive(Debug)]
pub struct SubscribeMessage {
    pub conn_id: Uuid,
    pub typ: SubscriberType,
}

#[derive(Debug)]
pub enum SubscriberType {
    All,
    LocalOnly,
}

#[derive(Debug)]
pub struct BroadcastPacketMessage {
    pub packet: Packet,
    pub source_conn_id: Option<Uuid>,
    pub local: bool,
}

#[derive(Debug)]
pub struct NewConnectionMessage {
    pub conn_id: Uuid,
    pub socket: TcpStream,
}

pub fn start(receiver: Receiver<MessengerOperations>) {
    let mut connection_map = HashMap::<Uuid, TcpStream>::new();
    let mut local_only_broadcast_list = HashSet::<Uuid>::new();
    let mut all_broadcast_list = HashSet::<Uuid>::new();
    let mut translation_data = HashMap::<Uuid, TranslationInfo>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            MessengerOperations::Send(msg) => {
                if let Some(socket) = connection_map.get(&msg.conn_id) {
                    let mut socket_clone = socket.try_clone().unwrap();
                    let translated_packet = match translation_data.get(&msg.conn_id) {
                        Some(translation_data) => translate_outgoing(msg.packet, *translation_data),
                        None => msg.packet,
                    };
                    write(&mut socket_clone, translated_packet);
                }
            }
            MessengerOperations::Broadcast(msg) => {
                (&all_broadcast_list).iter().for_each(|conn_id| {
                    if msg.source_conn_id.is_none() || msg.source_conn_id.unwrap() != *conn_id {
                        if let Some(socket) = connection_map.get(&conn_id) {
                            let mut socket_clone = socket.try_clone().unwrap();
                            let packet_clone = msg.packet.clone();
                            write(&mut socket_clone, packet_clone);
                        }
                    }
                });
                if msg.local {
                    (&local_only_broadcast_list).iter().for_each(|conn_id| {
                        if let Some(socket) = connection_map.get(&conn_id) {
                            let mut socket_clone = socket.try_clone().unwrap();
                            let packet_clone = msg.packet.clone();
                            write(&mut socket_clone, packet_clone);
                        }
                    });
                }
            }
            MessengerOperations::Subscribe(msg) => match msg.typ {
                SubscriberType::All => {
                    all_broadcast_list.insert(msg.conn_id);
                }
                SubscriberType::LocalOnly => {
                    local_only_broadcast_list.insert(msg.conn_id);
                }
            },
            MessengerOperations::New(msg) => {
                connection_map.insert(msg.conn_id, msg.socket);
            }
            MessengerOperations::UpdateTranslation(msg) => {
                translation_data.insert(
                    msg.conn_id,
                    TranslationInfo {
                        state: 0,
                        entity_id_block: 0,
                        map: msg.map,
                    },
                );
            }
        }
    }
}

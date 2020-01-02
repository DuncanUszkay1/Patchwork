use super::super::interfaces::messenger::{MessengerOperations, SubscriberType};
use super::packet::{translate_outgoing, write};
use super::translation::TranslationInfo;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use uuid::Uuid;

pub fn start(receiver: Receiver<MessengerOperations>) {
    let mut connection_map = HashMap::<Uuid, TcpStream>::new();
    let mut local_only_broadcast_list = HashSet::<Uuid>::new();
    let mut all_broadcast_list = HashSet::<Uuid>::new();
    let mut translation_data = HashMap::<Uuid, TranslationInfo>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            MessengerOperations::Send(msg) => {
                trace!(
                    "Sending packet {:?} to conn_id {:?}",
                    msg.packet.debug_print_type(),
                    msg.conn_id
                );
                if let Some(socket) = connection_map.get(&msg.conn_id) {
                    let mut socket_clone = socket.try_clone().unwrap();
                    let translated_packet = match translation_data.get(&msg.conn_id) {
                        Some(translation_data) => {
                            translate_outgoing(msg.packet, translation_data.clone())
                        }
                        None => msg.packet,
                    };
                    write(&mut socket_clone, translated_packet);
                    trace!("Send successful");
                } else {
                    trace!("Connection ID not found");
                }
            }
            MessengerOperations::Broadcast(msg) => {
                if msg.local {
                    trace!(
                        "Broadcasting packet {:?} from local source conn_id {:?}",
                        msg.packet.debug_print_type(),
                        msg.source_conn_id
                    );
                } else {
                    trace!(
                        "Broadcasting packet {:?} from remote source conn_id {:?}",
                        msg.packet.debug_print_type(),
                        msg.source_conn_id
                    );
                }
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
            MessengerOperations::Subscribe(msg) => {
                trace!(
                    "Subscribing conn_id {:?} with type {:?}",
                    msg.conn_id,
                    msg.typ
                );
                match msg.typ {
                    SubscriberType::All => {
                        all_broadcast_list.insert(msg.conn_id);
                    }
                    SubscriberType::LocalOnly => {
                        local_only_broadcast_list.insert(msg.conn_id);
                    }
                }
            }
            MessengerOperations::New(msg) => {
                trace!(
                    "New Connection with conn_id {:?} on socket {:?}",
                    msg.conn_id,
                    msg.socket
                );
                connection_map.insert(msg.conn_id, msg.socket);
            }
            MessengerOperations::UpdateTranslation(msg) => {
                trace!(
                    "Updating connection map for conn_id {:?} to {:?}",
                    msg.conn_id,
                    msg.map
                );
                translation_data.insert(
                    msg.conn_id,
                    TranslationInfo {
                        state: 0,
                        map: msg.map,
                    },
                );
            }
        }
    }
}

use super::super::interfaces::messenger::{MessengerOperations, SubscriberType};
use super::packet::{translate_outgoing, write, Packet};
use super::translation::TranslationInfo;

use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub fn start(receiver: Receiver<MessengerOperations>, _sender: Sender<MessengerOperations>) {
    let mut connection_map = HashMap::<Uuid, TcpStream>::new();
    let mut subscriber_list = SubscriberList::new();
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
                trace!(
                    "Broadcasting packet {:?} to subscriber_type {:?}",
                    msg.packet.debug_print_type(),
                    msg.subscriber_type,
                );
                let receipients: HashSet<Uuid> = subscriber_list.receipients(msg.subscriber_type);
                if let Some(source) = msg.source_conn_id {
                    let filtered_receipients: HashSet<Uuid> = receipients
                        .iter()
                        .filter(|conn_id| **conn_id != source)
                        .copied()
                        .collect();
                    broadcast(msg.packet, filtered_receipients, &connection_map)
                } else {
                    broadcast(msg.packet, receipients, &connection_map)
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
                        subscriber_list.add_local(msg.conn_id);
                        subscriber_list.add_remote(msg.conn_id);
                    }
                    SubscriberType::Local => {
                        subscriber_list.add_local(msg.conn_id);
                    }
                    SubscriberType::Remote => {
                        subscriber_list.add_remote(msg.conn_id);
                    }
                }
            }
            MessengerOperations::Close(msg) => {
                trace!("Closing connection {:?}", msg.conn_id);
                connection_map.remove(&msg.conn_id);
                translation_data.remove(&msg.conn_id);
                subscriber_list.remove(&msg.conn_id);
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

fn broadcast<'a, I: IntoIterator<Item = Uuid>>(
    packet: Packet,
    conn_ids: I,
    connection_map: &'a HashMap<Uuid, TcpStream>,
) {
    conn_ids.into_iter().for_each(|conn_id| {
        if let Some(socket) = connection_map.get(&conn_id) {
            let mut socket_clone = socket.try_clone().unwrap();
            let packet_clone = packet.clone();
            write(&mut socket_clone, packet_clone);
        }
    });
}

struct SubscriberList {
    remote_subscribers: HashSet<Uuid>,
    local_subscribers: HashSet<Uuid>,
}

impl SubscriberList {
    pub fn new() -> SubscriberList {
        SubscriberList {
            remote_subscribers: HashSet::<Uuid>::new(),
            local_subscribers: HashSet::<Uuid>::new(),
        }
    }

    pub fn receipients(&self, subscriber_type: SubscriberType) -> HashSet<Uuid> {
        match subscriber_type {
            SubscriberType::All => self
                .remote_subscribers
                .union(&self.local_subscribers)
                .copied()
                .collect(),
            SubscriberType::Local => self.local_subscribers.clone(),
            SubscriberType::Remote => self.remote_subscribers.clone(),
        }
    }

    pub fn add_local(&mut self, uuid: Uuid) {
        self.local_subscribers.insert(uuid);
    }

    pub fn add_remote(&mut self, uuid: Uuid) {
        self.remote_subscribers.insert(uuid);
    }

    pub fn remove(&mut self, uuid: &Uuid) {
        self.local_subscribers.remove(uuid);
        self.remote_subscribers.remove(uuid);
    }
}

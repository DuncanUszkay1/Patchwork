use super::packet::{write, Packet};
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
}

#[derive(Debug)]
pub struct SendPacketMessage {
    pub conn_id: Uuid,
    pub packet: Packet,
}

#[derive(Debug)]
pub struct SubscribeMessage {
    pub conn_id: Uuid,
    pub local: bool,
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
    // Connections that want all packets- including those from our peers
    let mut local_broadcast_list = HashSet::<Uuid>::new();
    // Connections that only want packets that involve events that occur on our server
    let mut broadcast_list = HashSet::<Uuid>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            MessengerOperations::Send(msg) => match connection_map.get(&msg.conn_id) {
                Some(socket) => {
                    let mut socket_clone = socket.try_clone().unwrap();
                    write(&mut socket_clone, msg.packet);
                }
                None => {}
            },
            MessengerOperations::Broadcast(msg) => {
                // Alright this local thing is confusing- we should think about renaming it. The
                // problem is local users (ones who connected to us directly) want to know about
                // our peers (so they want to know about non-local packets) and non-local users
                // only want to know about local packets
                let mut broadcast_count = 0;
                (&local_broadcast_list).iter().for_each(|conn_id| {
                    if msg.source_conn_id.is_none() || msg.source_conn_id.unwrap() != *conn_id {
                        if let Some(socket) = connection_map.get(&conn_id) {
                            broadcast_count += 1;
                            let mut socket_clone = socket.try_clone().unwrap();
                            let packet_clone = msg.packet.clone();
                            write(&mut socket_clone, packet_clone);
                        }
                    }
                });
                if msg.local {
                    (&broadcast_list).iter().for_each(|conn_id| {
                        if let Some(socket) = connection_map.get(&conn_id) {
                            broadcast_count += 1;
                            let mut socket_clone = socket.try_clone().unwrap();
                            let packet_clone = msg.packet.clone();
                            write(&mut socket_clone, packet_clone);
                        }
                    });
                }
            }
            MessengerOperations::Subscribe(msg) => {
                if msg.local {
                    local_broadcast_list.insert(msg.conn_id);
                } else {
                    broadcast_list.insert(msg.conn_id);
                }
            }
            MessengerOperations::New(msg) => {
                connection_map.insert(msg.conn_id, msg.socket);
            }
        }
    }
}

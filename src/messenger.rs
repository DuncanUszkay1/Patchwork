use super::keep_alive::{KeepAliveOperations, NewKeepAliveConnectionMessage};
use super::packet::{write, Packet};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
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
    ($messenger:expr, $packet:expr) => {
        $messenger.send(MessengerOperations::Broadcast(BroadcastPacketMessage {
            packet: $packet,
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
}

#[derive(Debug)]
pub struct BroadcastPacketMessage {
    pub packet: Packet,
}

#[derive(Debug)]
pub struct NewConnectionMessage {
    pub conn_id: Uuid,
    pub socket: TcpStream,
}

pub fn start_messenger(
    receiver: Receiver<MessengerOperations>,
    keep_alive_sender: Sender<KeepAliveOperations>,
) {
    let mut connection_map = HashMap::<Uuid, TcpStream>::new();
    let mut broadcast_list = Vec::<Uuid>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            MessengerOperations::Send(msg) => match connection_map.get(&msg.conn_id) {
                Some(socket) => {
                    let mut socket_clone = socket.try_clone().unwrap();
                    write(&mut socket_clone, msg.packet);
                }
                None => {
                    println!("messenger.rs failed to find the conn id {:?}", msg.conn_id);
                }
            },
            MessengerOperations::Broadcast(msg) => {
                (&broadcast_list).iter().for_each(|conn_id| {
                    //println!("broadcasting to {:?}", conn_id);
                    if let Some(socket) = connection_map.get(&conn_id) {
                        let mut socket_clone = socket.try_clone().unwrap();
                        let packet_clone = msg.packet.clone();
                        write(&mut socket_clone, packet_clone);
                    }
                });
            }
            MessengerOperations::Subscribe(msg) => broadcast_list.push(msg.conn_id),
            MessengerOperations::New(msg) => {
                connection_map.insert(msg.conn_id, msg.socket);
                keep_alive_sender
                    .send(KeepAliveOperations::New(NewKeepAliveConnectionMessage {
                        conn_id: msg.conn_id,
                    }))
                    .unwrap();
            }
        }
    }
}

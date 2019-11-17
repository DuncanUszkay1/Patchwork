use super::keep_alive::{KeepAliveOperations, NewKeepAliveConnectionMessage};
use super::packet::{write, Packet};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};

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
    New(NewConnectionMessage),
}

#[derive(Debug)]
pub struct SendPacketMessage {
    pub conn_id: u64,
    pub packet: Packet,
}

#[derive(Debug)]
pub struct BroadcastPacketMessage {
    pub packet: Packet,
}

#[derive(Debug)]
pub struct NewConnectionMessage {
    pub conn_id: u64,
    pub socket: TcpStream,
}

pub fn start_messenger(
    receiver: Receiver<MessengerOperations>,
    keep_alive_sender: Sender<KeepAliveOperations>,
) {
    let mut connection_map = HashMap::<u64, TcpStream>::new();

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
                connection_map.values().for_each(|socket| {
                    let mut socket_clone = socket.try_clone().unwrap();
                    let packet_clone = msg.packet.clone();
                    write(&mut socket_clone, packet_clone);
                });
            }
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

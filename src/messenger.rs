use super::packet::{write, Packet};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;

pub enum MessengerOperations {
    Send(SendPacketMessage),
    New(NewConnectionMessage),
}

#[derive(Debug)]
pub struct SendPacketMessage {
    pub conn_id: i32,
    pub packet: Packet,
}

#[derive(Debug)]
pub struct NewConnectionMessage {
    pub conn_id: i32,
    pub socket: TcpStream,
}

pub fn start_messenger(receiver: Receiver<MessengerOperations>) {
    let mut connection_map = HashMap::<i32, TcpStream>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            MessengerOperations::Send(msg) => match connection_map.get(&msg.conn_id) {
                Some(socket) => {
                    let mut socket_clone = socket.try_clone().unwrap();
                    write(&mut socket_clone, msg.packet);
                }
                None => {
                    println!(
                        "messenger.rs failed to find the conn id {:?}",
                        msg.conn_id
                    );
                }
            },
            MessengerOperations::New(msg) => {
                connection_map.insert(msg.conn_id, msg.socket);
            }
        }
    }
}

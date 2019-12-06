use super::messenger::{MessengerOperations, NewConnectionMessage};
use super::minecraft_protocol::read_var_int;

use std::env;
use std::io;
use std::io::{Cursor, Read};
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use super::packet_processor::{InboundPacketMessage, PacketProcessorOperations};
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub fn listen(
    inbound_packet_processor: Sender<PacketProcessorOperations>,
    messenger: Sender<MessengerOperations>,
) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", env::var("PORT").unwrap())).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let inbound_packet_processor_clone = inbound_packet_processor.clone();
        let messenger_clone = messenger.clone();
        let conn_id = Uuid::new_v4();
        thread::spawn(move || {
            handle_connection(
                stream,
                inbound_packet_processor_clone,
                messenger_clone,
                conn_id,
            );
        });
    }
}

pub fn handle_connection(
    mut stream: TcpStream,
    inbound_packet_processor: Sender<PacketProcessorOperations>,
    messenger: Sender<MessengerOperations>,
    conn_id: Uuid,
) {
    let stream_clone = stream.try_clone().unwrap();
    messenger
        .send(MessengerOperations::New(NewConnectionMessage {
            conn_id,
            socket: stream_clone,
        }))
        .unwrap();
    loop {
        match read_var_int(&mut stream) {
            Ok(length) => {
                let vec: Vec<u8> = (&stream)
                    .bytes()
                    .take(length as usize)
                    .map(|r: Result<u8, _>| {
                        r.expect("packet was smaller than length field indicated!")
                    })
                    .collect();
                let cursor = Cursor::new(vec);
                inbound_packet_processor
                    .send(PacketProcessorOperations::Inbound(InboundPacketMessage {
                        conn_id,
                        cursor,
                    }))
                    .expect("Inbound packet processor crashed");
            }
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof => {}
                    _ => {
                        panic!("conn closed due to {:?}", e);
                    }
                };
                break;
            }
        }
    }
}

pub fn new_connection(peer_address: String, peer_port: u16) -> Result<TcpStream, io::Error> {
    let peer_info = format!("{}:{}", peer_address, peer_port.to_string());
    TcpStream::connect(peer_info)
}

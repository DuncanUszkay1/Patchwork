use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::PacketProcessor;

use super::models::minecraft_protocol::MinecraftProtocolReader;

use std::env;
use std::io::ErrorKind::UnexpectedEof;
use std::io::{Cursor, Error, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;

use uuid::Uuid;

pub fn listen<
    M: 'static + Messenger + Clone + Send,
    PP: 'static + PacketProcessor + Clone + Send,
>(
    inbound_packet_processor: PP,
    messenger: M,
) {
    let connection_string = format!("127.0.0.1:{}", env::var("PORT").unwrap());
    let listener = TcpListener::bind(connection_string.clone()).unwrap();

    trace!("Listening on {:?}", connection_string);

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

pub fn handle_connection<M: Messenger, PP: PacketProcessor>(
    mut stream: TcpStream,
    inbound_packet_processor: PP,
    messenger: M,
    conn_id: Uuid,
) {
    let stream_clone = stream.try_clone().unwrap();
    messenger.new_connection(conn_id, stream_clone);
    loop {
        match stream.try_read_var_int() {
            Ok(length) => {
                let vec: Vec<u8> = (&stream)
                    .bytes()
                    .take(length as usize)
                    .map(|r: Result<u8, _>| {
                        r.expect("packet was smaller than length field indicated!")
                    })
                    .collect();
                let cursor = Cursor::new(vec);
                inbound_packet_processor.inbound(conn_id, cursor);
            }
            Err(e) => {
                match e.kind() {
                    UnexpectedEof => {}
                    _ => {
                        panic!("conn closed due to {:?}", e);
                    }
                };
                break;
            }
        }
    }
}

pub fn new_connection(peer_address: String, peer_port: u16) -> Result<TcpStream, Error> {
    let peer_info = format!("{}:{}", peer_address, peer_port.to_string());
    TcpStream::connect(peer_info)
}

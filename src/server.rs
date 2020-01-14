use super::interfaces::connection::ConnectionService;
use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::PacketProcessor;

use super::models::minecraft_protocol::MinecraftProtocolReader;

use std::env;
use std::io::ErrorKind::{ConnectionReset, UnexpectedEof};
use std::io::{Cursor, Error, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::sleep;
use std::time;

use uuid::Uuid;

pub fn listen<
    M: 'static + Messenger + Clone + Send,
    PP: 'static + PacketProcessor + Clone + Send,
    CS: 'static + ConnectionService + Clone + Send,
>(
    inbound_packet_processor: PP,
    connection_service: CS,
    messenger: M,
) {
    let connection_string = format!("127.0.0.1:{}", env::var("PORT").unwrap());
    let listener = TcpListener::bind(connection_string.clone()).unwrap();

    trace!("Listening on {:?}", connection_string);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let inbound_packet_processor_clone = inbound_packet_processor.clone();
        let messenger_clone = messenger.clone();
        let closure_connection_service = connection_service.clone();
        let conn_id = Uuid::new_v4();
        thread::spawn(move || {
            handle_connection(
                stream,
                inbound_packet_processor_clone,
                messenger_clone,
                conn_id,
                || closure_connection_service.close(conn_id),
            );
        });
    }
}

pub fn handle_connection<M: Messenger, PP: PacketProcessor, F: Fn()>(
    mut stream: TcpStream,
    inbound_packet_processor: PP,
    messenger: M,
    conn_id: Uuid,
    on_closure: F,
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
                    UnexpectedEof => on_closure(),
                    ConnectionReset => on_closure(),
                    _ => {
                        panic!("conn closed due to {:?}", e);
                    }
                };
                break;
            }
        }
    }
}

//Just doing a simple linear backoff for now, probably want something a little more sophisticated
//eventually
pub fn wait_for_connection<F: FnOnce(TcpStream)>(
    peer_address: String,
    peer_port: u16,
    on_connection: F,
) {
    let backoff = 1;
    loop {
        if let Ok(connection) = new_connection(peer_address.clone(), peer_port) {
            trace!("Connection Established");
            on_connection(connection);
            break;
        } else {
            let backoff = if backoff < 10 { backoff + 1 } else { backoff };
            trace!("Failed to connect- retrying in {:?}s", backoff);
            sleep(time::Duration::from_secs(backoff));
        }
    }
}

pub fn new_connection(peer_address: String, peer_port: u16) -> Result<TcpStream, Error> {
    let peer_info = format!("{}:{}", peer_address, peer_port.to_string());
    TcpStream::connect(peer_info)
}

use super::packet;
use super::packet::{write, Packet};
use std::net::TcpStream;

const FAKE_RESPONSE: &str = "{\"version\": {\"name\": \"1.13.2\",\"protocol\": 404},\"players\": {\"max\": 100,\"online\": 5,\"sample\": [{\"name\": \"thinkofdeath\",\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\": {\"text\": \"Hello world\"},\"favicon\": \"data:image/png;base64,<data>\"}";

// Called when client pings the server
pub fn init_client_ping(p: Packet, state: &mut u64, stream: &mut TcpStream) {
    println!("Ping packet : {:?}",p);

    match p.clone() {
        Packet::StatusRequest(_) => {
            let status_response = packet::StatusResponse {
                json_response: String::from(FAKE_RESPONSE),
            };
            write(stream, Packet::StatusResponse(status_response));
        }
        Packet::Ping(ping) => {
            let pong = packet::Pong {
                payload: ping.payload,
            };
            write(stream, Packet::Pong(pong));
        }
        _ => {}
    }
}
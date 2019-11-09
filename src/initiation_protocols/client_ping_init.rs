use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet;
use super::packet::Packet;
use std::sync::mpsc::Sender;

const FAKE_RESPONSE: &str = "{\"version\": {\"name\": \"1.13.2\",\"protocol\": 404},\"players\": {\"max\": 100,\"online\": 5,\"sample\": [{\"name\": \"thinkofdeath\",\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\": {\"text\": \"Hello world\"},\"favicon\": \"data:image/png;base64,<data>\"}";

// Called when client pings the server
pub fn init_client_ping(p: Packet, conn_id: u64, messenger: Sender<MessengerOperations>) {
    println!("Ping packet : {:?}", p);

    match p.clone() {
        Packet::StatusRequest(_) => {
            let status_response = packet::StatusResponse {
                json_response: String::from(FAKE_RESPONSE),
            };

            send_packet!(messenger, conn_id, Packet::StatusResponse(status_response)).unwrap();
        }
        Packet::Ping(ping) => {
            let pong = packet::Pong {
                payload: ping.payload,
            };
            send_packet!(messenger, conn_id, Packet::Pong(pong)).unwrap();
        }
        _ => {}
    }
}

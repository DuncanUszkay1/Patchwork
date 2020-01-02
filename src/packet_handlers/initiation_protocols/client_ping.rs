use super::interfaces::messenger::Messenger;
use super::packet;
use super::packet::Packet;
use super::translation::TranslationUpdates;
use uuid::Uuid;

const FAKE_RESPONSE: &str = "{\"version\": {\"name\": \"1.13.2\",\"protocol\": 404},\"players\": {\"max\": 100,\"online\": 5,\"sample\": [{\"name\": \"thinkofdeath\",\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\": {\"text\": \"Hello world\"},\"favicon\": \"data:image/png;base64,<data>\"}";

// Called when client pings the server
pub fn handle_client_ping_packet<M: Messenger>(
    p: Packet,
    conn_id: Uuid,
    messenger: M,
) -> TranslationUpdates {
    match p.clone() {
        Packet::StatusRequest(_) => {
            let status_response = packet::StatusResponse {
                json_response: String::from(FAKE_RESPONSE),
            };

            messenger.send_packet(conn_id, Packet::StatusResponse(status_response));
        }
        Packet::Ping(ping) => {
            let pong = packet::Pong {
                payload: ping.payload,
            };
            messenger.send_packet(conn_id, Packet::Pong(pong));
        }
        _ => {}
    }
    TranslationUpdates::NoChange
}

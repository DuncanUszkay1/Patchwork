use super::constants::{SERVER_DESCRIPTION, SERVER_PROTOCOL, SERVER_VERSION};
use super::interfaces::messenger::Messenger;
use super::interfaces::player::PlayerState;
use super::minecraft_types::{Description, Version};
use super::packet;
use super::packet::Packet;
use super::translation::TranslationUpdates;
use uuid::Uuid;

// Called when client pings the server
pub fn handle_client_ping_packet<M: Messenger, P: PlayerState>(
    p: Packet,
    conn_id: Uuid,
    messenger: M,
    player_state: P,
) -> TranslationUpdates {
    match p {
        Packet::StatusRequest(_) => {
            let version = Version {
                name: SERVER_VERSION.to_string(),
                protocol: SERVER_PROTOCOL,
            };
            let description = Description {
                text: SERVER_DESCRIPTION.to_string(),
            };

            player_state.status_response(conn_id, version, description);
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

use super::game_state::player::{Angle, NewPlayerMessage, Player, PlayerStateOperations, Position};
use super::packet::Packet;
use super::TranslationUpdates;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub fn border_cross_login(
    p: Packet,
    conn_id: Uuid,
    player_state: Sender<PlayerStateOperations>,
) -> TranslationUpdates {
    match p {
        Packet::PlayerPositionAndLook(packet) => {
            let player = Player {
                conn_id,
                uuid: Uuid::new_v4(),
                name: String::from("ghost"),
                entity_id: 0, // replaced by player state
                position: Position {
                    x: packet.x,
                    y: packet.feet_y,
                    z: packet.z,
                },
                angle: Angle {
                    pitch: packet.pitch,
                    yaw: packet.yaw,
                },
            };

            //update the gamestate with this new player
            player_state
                .send(PlayerStateOperations::New(NewPlayerMessage {
                    conn_id,
                    player,
                }))
                .unwrap();
            TranslationUpdates::State(3)
        }
        _ => TranslationUpdates::NoChange,
    }
}

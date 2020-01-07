use super::interfaces::player::{Angle, Player, PlayerState, Position};
use super::packet::Packet;
use super::translation::TranslationUpdates;
use uuid::Uuid;

pub fn border_cross_login<P: PlayerState>(
    p: Packet,
    conn_id: Uuid,
    player_state: P,
) -> TranslationUpdates {
    match p {
        Packet::BorderCrossLogin(packet) => {
            let player = Player {
                conn_id,
                uuid: Uuid::new_v4(),
                name: packet.username,
                //Hardcoded to assume that 950-1000 is the range used for this peer's anchors
                entity_id: 950 + packet.entity_id,
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
            player_state.new_player(conn_id, player);
            TranslationUpdates::State(3)
        }
        _ => TranslationUpdates::NoChange,
    }
}

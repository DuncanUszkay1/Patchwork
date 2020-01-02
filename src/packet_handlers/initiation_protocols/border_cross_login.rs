use super::game_state::player::{Angle, Player, PlayerState, Position};
use super::packet::Packet;
use super::translation::TranslationUpdates;
use uuid::Uuid;

pub fn border_cross_login<P: PlayerState>(
    p: Packet,
    conn_id: Uuid,
    player_state: P,
) -> TranslationUpdates {
    match p {
        Packet::PlayerPositionAndLook(packet) => {
            let player = Player {
                conn_id,
                uuid: Uuid::new_v4(),
                name: String::from("ghost"),
                // hard coded to only work for the first player to login
                // need to augment this packet to include the entity id on the host peer for this
                // to work
                entity_id: 950,
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

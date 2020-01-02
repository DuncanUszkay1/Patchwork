use super::game_state::player::{Angle, PlayerState, Position};
use super::packet::Packet;
use uuid::Uuid;

pub fn route_packet<P: PlayerState>(p: Packet, conn_id: Uuid, player_state: P) {
    match p {
        Packet::PlayerPosition(player_position) => {
            player_state.move_and_look(
                conn_id,
                Some(Position {
                    x: player_position.x,
                    y: player_position.feet_y,
                    z: player_position.z,
                }),
                None,
            );
        }
        Packet::PlayerPositionAndLook(player_position_and_look) => {
            player_state.move_and_look(
                conn_id,
                Some(Position {
                    x: player_position_and_look.x,
                    y: player_position_and_look.feet_y,
                    z: player_position_and_look.z,
                }),
                Some(Angle {
                    yaw: player_position_and_look.yaw,
                    pitch: player_position_and_look.pitch,
                }),
            );
        }
        Packet::PlayerLook(player_look) => {
            player_state.move_and_look(
                conn_id,
                None,
                Some(Angle {
                    yaw: player_look.yaw,
                    pitch: player_look.pitch,
                }),
            );
        }
        Packet::Unknown => (),
        _ => {
            panic!("Gameplay router received unexpected packet {:?}", p);
        }
    }
}

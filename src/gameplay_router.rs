use super::game_state::player::{
    Angle, PlayerLookMessage, PlayerMoveAndLookMessage, PlayerMovementMessage,
    PlayerStateOperations, Position,
};
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub fn route_packet(p: Packet, conn_id: Uuid, player_state: Sender<PlayerStateOperations>) {
    match p {
        Packet::PlayerPosition(player_position) => {
            player_state
                .send(PlayerStateOperations::Move(PlayerMovementMessage {
                    conn_id,
                    new_position: Position {
                        x: player_position.x,
                        y: player_position.feet_y,
                        z: player_position.z,
                    },
                }))
                .unwrap();
        }
        Packet::PlayerPositionAndLook(player_position_and_look) => {
            player_state
                .send(PlayerStateOperations::MoveAndLook(
                    PlayerMoveAndLookMessage {
                        conn_id,
                        new_position: Position {
                            x: player_position_and_look.x,
                            y: player_position_and_look.feet_y,
                            z: player_position_and_look.z,
                        },
                        new_angle: Angle {
                            yaw: player_position_and_look.yaw,
                            pitch: player_position_and_look.pitch,
                        },
                    },
                ))
                .unwrap();
        }
        Packet::PlayerLook(player_look) => {
            player_state
                .send(PlayerStateOperations::Look(PlayerLookMessage {
                    conn_id,
                    new_angle: Angle {
                        yaw: player_look.yaw,
                        pitch: player_look.pitch,
                    },
                }))
                .unwrap();
        }
        Packet::Unknown => (),
        _ => {
            panic!("Gameplay router received unexpected packet {:?}", p);
        }
    }
}

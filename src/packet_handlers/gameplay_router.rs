use super::chat_message_router;
use super::interfaces::block::BlockState;
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::{Angle, PlayerState, Position};
use super::packet::Packet;
use uuid::Uuid;

pub fn route_packet<P: PlayerState, PA: PatchworkState, B: BlockState>(
    p: Packet,
    conn_id: Uuid,
    player_state: P,
    patchwork_state: PA,
    block_state: B,
) {
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
        Packet::ChatMessage(_) => {
            chat_message_router::route_packet(p, conn_id, patchwork_state);
        }
        Packet::PlayerDigging(block_packet) => {
            println!("PlayerDigging from gameplay_router");
            block_state.break_block_serverbound(conn_id, block_packet);
        }
        Packet::BlockChange(block_packet) => {
            println!("BlockChange from gameplay_router");
            block_state.break_block_clientbound(conn_id, block_packet);
        }
        Packet::Unknown => (),
        _ => {
            panic!("Gameplay router received unexpected packet {:?}", p);
        }
    }
}

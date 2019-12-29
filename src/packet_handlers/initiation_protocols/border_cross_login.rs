#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::game_state::block;
use super::game_state::block::BlockStateOperations;
use super::game_state::patchwork::PatchworkStateOperations;
use super::game_state::player;
use super::game_state::player::{Angle, NewPlayerMessage, Player, PlayerStateOperations, Position};
use super::messenger::{MessengerOperations, SendPacketMessage, SubscribeMessage, SubscriberType};
use super::packet;
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub fn border_cross_login(
    p: Packet,
    conn_id: Uuid,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
) -> bool {
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
            true
        }
        _ => false,
    }
}

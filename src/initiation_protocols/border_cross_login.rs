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

// Called by the server when a new player is walking in its map
pub fn init_border_cross_login(
    p: Packet,
    conn_id: Uuid,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
) {
    let player = Player {
        conn_id,
        uuid: Uuid::new_v4(),
        name: String::from("ghost"),
        entity_id: 0, // replaced by player state
        position: Position {
            x: 5.0,
            y: 16.0,
            z: 5.0,
        },
        angle: Angle {
            pitch: 0.0,
            yaw: 0.0,
        },
    };

    //update the gamestate with this new player
    player_state
        .send(PlayerStateOperations::New(NewPlayerMessage {
            conn_id,
            player,
        }))
        .unwrap();
}

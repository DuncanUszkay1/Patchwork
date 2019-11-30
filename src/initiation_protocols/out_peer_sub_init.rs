#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::game_state::block;
use super::game_state::block::BlockStateOperations;
use super::game_state::player;
use super::game_state::player::{NewPlayerMessage, Player, PlayerStateOperations, Position};
use super::packet::{EntityLookAndMove, Packet, PlayerInfo, SpawnPlayer};
use std::sync::mpsc::Sender;
use uuid::Uuid;

// Called when requesting a peer subscription to another server
pub fn init_outgoing_peer_sub(
    p: Packet,
    conn_id: Uuid,
    player_state: Sender<PlayerStateOperations>,
    block_state: Sender<BlockStateOperations>,
) {
    //report current state to player (soon to be in it's own component for reuse)
    //the only state we keep right now is players
    player_state
        .send(PlayerStateOperations::Report(player::ReportMessage {
            conn_id,
        }))
        .unwrap();

    block_state
        .send(BlockStateOperations::Report(block::ReportMessage {
            conn_id,
        }))
        .unwrap();
}

use super::messenger::{
    BroadcastPacketMessage, MessengerOperations, SubscribeMessage, SubscriberType,
};
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

use super::game_state::block;
use super::game_state::block::BlockStateOperations;
use super::game_state::player;
use super::game_state::player::PlayerStateOperations;

pub fn handle_peer_packet(packet: Packet, messenger: Sender<MessengerOperations>) {
    //Whenever a peer we subscribe to sends us a packet, we just broadcast it to all our local
    //players
    broadcast_packet!(messenger, packet, None, false).unwrap();
}

pub fn handle_subscriber_packet(
    conn_id: Uuid,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
    block_state: Sender<BlockStateOperations>,
) {
    //Everytime a subscriber sends us a packet, we subscribe them to our messages and report our
    //state to them

    messenger
        .send(MessengerOperations::Subscribe(SubscribeMessage {
            conn_id,
            typ: SubscriberType::LocalOnly,
        }))
        .unwrap();

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

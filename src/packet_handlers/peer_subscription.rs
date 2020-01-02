use super::messenger::{Messenger, SubscriberType};
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

use super::game_state::block;
use super::game_state::block::BlockStateOperations;
use super::game_state::player;
use super::game_state::player::PlayerStateOperations;

pub fn handle_peer_packet<M: Messenger>(
    packet: Packet,
    messenger: M,
    player_state: Sender<PlayerStateOperations>,
) {
    match packet.clone() {
        Packet::SpawnPlayer(packet) => {
            if packet.entity_id >= 1000 {
                messenger.broadcast_packet(Packet::SpawnPlayer(packet), None, false);
            }
        }
        //We really don't want to have to do this for every type of packet that has an entity id
        //There's probably a better solution here, a macro might do it since the code should be
        //identical but then we still need to list all the packets we can get from the peer that
        //have an entity id in them
        Packet::EntityLookAndMove(packet) => {
            let entity_id = packet.entity_id;
            source_anchored_event(Packet::EntityLookAndMove(packet), entity_id, player_state);
        }
        _ => {
            messenger.broadcast_packet(packet, None, false);
        }
    }
}

fn source_anchored_event(
    packet: Packet,
    entity_id: i32,
    player_state: Sender<PlayerStateOperations>,
) {
    player_state
        .send(PlayerStateOperations::BroadcastAnchoredEvent(
            player::BroadcastAnchoredEventMessage { entity_id, packet },
        ))
        .unwrap();
}

pub fn handle_subscriber_packet<M: Messenger>(
    conn_id: Uuid,
    messenger: M,
    player_state: Sender<PlayerStateOperations>,
    block_state: Sender<BlockStateOperations>,
) {
    //Everytime a subscriber sends us a packet, we subscribe them to our messages and report our
    //state to them

    messenger.subscribe(conn_id, SubscriberType::LocalOnly);

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

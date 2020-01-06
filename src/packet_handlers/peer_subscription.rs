use super::interfaces::messenger::{Messenger, SubscriberType};
use super::packet::Packet;
use uuid::Uuid;

use super::interfaces::block::BlockState;
use super::interfaces::player::PlayerState;

pub fn handle_peer_packet<M: Messenger, P: PlayerState>(
    packet: Packet,
    messenger: M,
    player_state: P,
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
            player_state.broadcast_anchored_event(entity_id, Packet::EntityLookAndMove(packet));
        }
        _ => {
            messenger.broadcast_packet(packet, None, false);
        }
    }
}

pub fn handle_subscriber_packet<M: Messenger, P: PlayerState, B: BlockState>(
    conn_id: Uuid,
    messenger: M,
    player_state: P,
    block_state: B,
) {
    //Everytime a subscriber sends us a packet, we subscribe them to our messages and report our
    //state to them

    trace!("Reporting state to peer");

    messenger.subscribe(conn_id, SubscriberType::LocalOnly);
    player_state.report(conn_id);
    block_state.report(conn_id);
}

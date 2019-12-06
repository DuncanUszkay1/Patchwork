use super::game_state::block::BlockStateOperations;
use super::game_state::patchwork::PatchworkStateOperations;
use super::game_state::player::PlayerStateOperations;
use super::gameplay_router;
use super::initiation_protocols::{
    border_cross_login_init, client_ping_init, handshake_init, in_peer_sub_init, login_init,
    out_peer_sub_init,
};
use super::messenger::MessengerOperations;
use super::packet::Packet;
use super::packet_processor::TranslationUpdates;
use std::sync::mpsc::Sender;
use uuid::Uuid;

// Routes the packet to the corresponding service according to the connection state
pub fn route_packet(
    packet: Packet,
    state: i32,
    conn_id: Uuid,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
    block_state: Sender<BlockStateOperations>,
    patchwork_state: Sender<PatchworkStateOperations>,
) -> TranslationUpdates {
    let st = Status::value(state);
    match st {
        Status::Handshake => TranslationUpdates::State(handshake_init::init_handshake(packet)),
        Status::Login => {
            login_init::init_login(
                packet,
                conn_id,
                messenger,
                player_state,
                block_state,
                patchwork_state,
            );
            TranslationUpdates::State(3)
        }
        Status::ClientPing => {
            client_ping_init::init_client_ping(packet, conn_id, messenger);
            TranslationUpdates::NoChange
        }
        Status::Play => {
            gameplay_router::route_packet(packet, conn_id, player_state);
            TranslationUpdates::NoChange
        }
        Status::BorderCrossLogin => {
            border_cross_login_init::init_border_cross_login(packet);
            TranslationUpdates::NoChange
        }
        Status::InPeerSub => {
            in_peer_sub_init::init_incoming_peer_sub(packet, conn_id, messenger);
            TranslationUpdates::NoChange
        }
        Status::OutPeerSub => {
            out_peer_sub_init::init_outgoing_peer_sub(
                packet,
                conn_id,
                messenger,
                player_state,
                block_state,
            );
            TranslationUpdates::NoChange
        }
    }
}

enum Status {
    Handshake,
    ClientPing,
    Login,
    Play,
    BorderCrossLogin,
    InPeerSub,
    OutPeerSub,
}

impl Status {
    fn value(status: i32) -> Status {
        match status {
            0 => Status::Handshake,
            1 => Status::ClientPing,
            2 => Status::Login,
            3 => Status::Play,
            4 => Status::BorderCrossLogin,
            5 => Status::InPeerSub,
            6 => Status::OutPeerSub,
            _ => panic!("Bad state"),
        }
    }
}

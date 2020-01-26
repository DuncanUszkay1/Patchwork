use super::interfaces::block::BlockState;
use super::interfaces::messenger::Messenger;
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::PlayerState;

use super::initiation_protocols::{border_cross_login, client_ping, handshake, login};
use super::packet::Packet;
use super::peer_subscription;
use super::translation::TranslationUpdates;
use uuid::Uuid;

// Routes the packet to the corresponding service according to the connection state
pub fn route_packet<
    M: Messenger + Clone,
    P: PlayerState + Clone,
    PA: PatchworkState + Clone,
    B: BlockState + Clone,
>(
    packet: Packet,
    state: i32,
    conn_id: Uuid,
    messenger: M,
    player_state: P,
    block_state: B,
    patchwork_state: PA,
) -> TranslationUpdates {
    let st = Status::from_i32(state);
    match st {
        Status::Handshake => handshake::handle_handshake_packet(packet),
        Status::Login => login::handle_login_packet(
            packet,
            conn_id,
            messenger,
            player_state,
            block_state,
            patchwork_state,
        ),
        Status::ClientPing => {
            client_ping::handle_client_ping_packet(packet, conn_id, messenger, player_state)
        }
        Status::Play => {
            patchwork_state.route_player_packet(packet, conn_id);
            TranslationUpdates::NoChange
        }
        Status::BorderCrossLogin => {
            border_cross_login::border_cross_login(packet, conn_id, player_state)
        }
        Status::InPeerSub => {
            peer_subscription::handle_peer_packet(packet, messenger, player_state);
            TranslationUpdates::NoChange
        }
        Status::OutPeerSub => {
            peer_subscription::handle_subscriber_packet(
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
    fn from_i32(status: i32) -> Status {
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

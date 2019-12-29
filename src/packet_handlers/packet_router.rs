use super::game_state::block::BlockStateOperations;
use super::game_state::patchwork::{PatchworkStateOperations, RouteMessage};
use super::game_state::player::PlayerStateOperations;
use super::initiation_protocols::{border_cross_login, client_ping, handshake, login};
use super::messenger::MessengerOperations;
use super::packet::Packet;
use super::peer_subscription;
use super::translation::TranslationUpdates;
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
        Status::ClientPing => client_ping::handle_client_ping_packet(packet, conn_id, messenger),
        Status::Play => {
            patchwork_state
                .send(PatchworkStateOperations::RoutePlayerPacket(RouteMessage {
                    packet: packet.clone(),
                    conn_id,
                }))
                .unwrap();
            TranslationUpdates::NoChange
        }
        Status::BorderCrossLogin => {
            border_cross_login::border_cross_login(packet, conn_id, player_state)
        }
        Status::InPeerSub => {
            peer_subscription::handle_peer_packet(packet, messenger);
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

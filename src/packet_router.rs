use super::game_state::player::PlayerStateOperations;
use super::initiation_protocols::{
    border_cross_login_init, client_ping_init, handshake_init, in_peer_sub_init, login_init,
    out_peer_sub_init,
};
use super::messenger::MessengerOperations;
use super::packet::Packet;
use std::sync::mpsc::Sender;

// Routes the packet to the corresponding service according to the connection state
pub fn route_packet(
    p: Packet,
    state: &mut u64,
    conn_id: u64,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
) {
    let st = Status::value(*state);
    match st {
        Status::Handshake => handshake_init::init_handshake(p, state),
        Status::ClientPing => client_ping_init::init_client_ping(p, conn_id, messenger),
        Status::Login => login_init::init_login(p, state, conn_id, messenger, player_state),
        Status::Play => (),
        Status::BorderCrossLogin => border_cross_login_init::init_border_cross_login(p, state),
        Status::InPeerSub => in_peer_sub_init::init_incoming_peer_sub(p, state),
        Status::OutPeerSub => out_peer_sub_init::init_outgoing_peer_sub(p, state),
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
    fn value(status: u64) -> Status {
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

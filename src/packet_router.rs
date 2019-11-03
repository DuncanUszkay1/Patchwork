use super::game_state;
use super::game_state::play;
use super::messenger;
use super::messenger::MessengerOperations;
use super::packet;
use super::packet::Packet;
use std::sync::mpsc::Sender;

mod border_cross_login_init;
mod client_ping_init;
mod handshake_init;
mod in_peer_sub_init;
mod login_init;
mod out_peer_sub_init;

// Routes the packet to the corresponding service according to the connection state
pub fn route_packet(
    p: Packet,
    state: &mut u64,
    conn_id: i32,
    messenger: Sender<MessengerOperations>,
) {
    let st = Status::value(*state);
    match st {
        Status::Handshake => handshake_init::init_handshake(p, state),
        Status::ClientPing => client_ping_init::init_client_ping(p, conn_id, messenger),
        Status::Login => login_init::init_login(p, state, conn_id, messenger),
        Status::Play => game_state::play(p, conn_id, messenger),
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

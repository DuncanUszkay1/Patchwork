use super::packet;
use super::packet::Packet;
use std::net::TcpStream;

mod handshake_init;
mod client_ping_init;
mod login_init;
mod border_cross_login_init;
mod in_peer_sub_init;
mod out_peer_sub_init;

// Routes the packet to the corresponding service according to the connection state
pub fn route_packet(p: Packet, state: &mut u64, stream: &mut TcpStream) {
    let st = Status::value(*state);
    match st {
        Status::Handshake => handshake_init::init_handshake(p, state),
        Status::ClientPing => client_ping_init::init_client_ping(p, state, stream),
        Status::Login => login_init::init_login(p, state),
        Status::BorderCrossLogin => border_cross_login_init::init_border_cross_login(p, state),
        Status::InPeerSub => in_peer_sub_init::init_incoming_peer_sub(p, state),
        Status::OutPeerSub => out_peer_sub_init::init_outgoing_peer_sub(p, state),
    }
}

enum Status {
    Handshake,
    ClientPing,
    Login,
    BorderCrossLogin,
    InPeerSub,
    OutPeerSub,
}

impl Status {
    fn value(status:u64) -> Status {
        match status {
            0 => Status::Handshake,
            1 => Status::ClientPing,
            2 => Status::Login,
            3 => Status::BorderCrossLogin,
            4 => Status::InPeerSub,
            5 => Status::OutPeerSub,
            _ => panic!("Bad state"),
        }
    }
}
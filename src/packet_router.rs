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
    match state {
        0 => handshake_init::init_handshake(p, state),
        1 => client_ping_init::init_client_ping(p, state, stream),
        2 => login_init::init_login(p, state),
        3 => border_cross_login_init::init_border_cross_login(p, state),
        4 => in_peer_sub_init::init_incoming_peer_sub(p, state),
        5 => out_peer_sub_init::init_outgoing_peer_sub(p, state),
        _ => panic!("Bad state"),
    }
}
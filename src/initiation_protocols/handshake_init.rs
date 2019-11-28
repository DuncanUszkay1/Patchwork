use super::packet::Packet;

// Called upon handshake
pub fn init_handshake(p: Packet) -> i32 {
    match p.clone() {
        Packet::Handshake(handshake) => handshake.next_state,
        _ => panic!("Invalid packet (handshake)"),
    }
}

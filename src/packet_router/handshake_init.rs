use super::packet::Packet;

// Called upon handshake between two servers
pub fn init_handshake(p: Packet, state: &mut u64) {
    println!("Handshake packet: {:?}", p);

    *state = match p.clone() {
        Packet::Handshake(handshake) => handshake.next_state,
        _ => *state,
    };
}

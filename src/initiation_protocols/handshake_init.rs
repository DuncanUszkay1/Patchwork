use super::packet::Packet;

// Called upon handshake
pub fn init_handshake(p: Packet, state: &mut i32) {
    println!("Handshake packet: {:?}", p);

    *state = match p.clone() {
        Packet::Handshake(handshake) => handshake.next_state,
        _ => {
            println!("Invalid packet (handshake)");
            *state
        }
    };
}

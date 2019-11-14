use super::packet::Packet;
use std::collections::HashMap;

// Called upon handshake
pub fn init_handshake(p: Packet, state: &mut u64, peer_map: &mut HashMap::<u64, String>, mut next_peer_id:u64) {
    println!("Handshake packet: {:?}", p);

    *state = match p.clone() {
        Packet::Handshake(handshake) => handshake.next_state,
        Packet::P2PHandshake(p2p_hs) => {
            peer_map.insert(next_peer_id, p2p_hs.peer);
            next_peer_id+=1;
            p2p_hs.next_state
        },
        _ => {
            println!("Invalid packet (handshake)");
            *state
        }
    };
}

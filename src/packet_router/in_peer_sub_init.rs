#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::packet;
use super::packet::{read, write, Packet};

// Called when a server requests a peer subscription
pub fn init_incoming_peer_sub(p: Packet, state: &mut u64) {
    println!("TODO");
}

#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::packet;
use super::packet::{read, write, Packet};

// Called when requesting a peer subscription to another server
pub fn init_outgoing_peer_sub(p: Packet, state: &mut u64) {
    println!("TODO");
}
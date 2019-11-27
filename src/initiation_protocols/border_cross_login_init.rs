#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::packet;
use super::packet::{read, write, Packet};

// Called by the server when a new player is walking in its map
pub fn init_border_cross_login(p: Packet, state: &mut i32) {
    unimplemented!("TODO");
}

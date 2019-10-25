#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::packet;
use super::packet::{read, write, Packet};

// Called upon user login
pub fn init_login(p: Packet, state: &mut u64) {
    println!("TODO");
}

//  For a packet handler to be an initiation protocol that means that the logic of the method is such
//  that eventually the state of the connection will change such that this method won't be called
//  again for this connection
//
//  Example: login protocol which transitions into the play state once sufficient information has been passed
//  between server and client

pub mod border_cross_login;
pub mod client_ping;
pub mod handshake;
pub mod login;

use super::game_state;
use super::messenger;
use super::packet;
use super::translation;

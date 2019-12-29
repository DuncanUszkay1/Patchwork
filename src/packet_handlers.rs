// For something to be a packet handler the following must be true:
//    - the module contains a method which is to be called upon receiving a packet
//
// Example: the gameplay router provides the route_packet method which handles packets from clients
// in the play state

pub mod gameplay_router;
pub mod initiation_protocols;
pub mod packet_router;
pub mod peer_subscription;

use super::services::game_state;
use super::services::messenger;

use super::models::packet;
use super::models::translation;

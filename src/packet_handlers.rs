// For something to be a packet handler the following must be true:
//    - the module contains a method which is to be called upon receiving a packet
//
// Example: the gameplay router provides the route_packet method which handles packets from clients
// in the play state

pub mod gameplay_router;
pub mod initiation_protocols;
pub mod peer_subscription;

use super::game_state;
use super::messenger;
use super::packet;
use super::translation::TranslationUpdates;

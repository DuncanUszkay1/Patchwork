#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::messenger::{
    BroadcastPacketMessage, MessengerOperations, NewConnectionMessage, SendPacketMessage,
};
use super::packet;
use super::packet::{EntityLookAndMove, Packet, PlayerInfo, SpawnPlayer};
use std::sync::mpsc::Sender;
use uuid::Uuid;

// needs to be renamed, just didn't want to make another one of these
pub fn init_incoming_peer_sub(p: Packet, conn_id: Uuid, messenger: Sender<MessengerOperations>) {
    broadcast_packet!(messenger, p).unwrap();
}

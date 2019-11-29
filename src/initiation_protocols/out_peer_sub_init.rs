#![allow(unused)] // removing warnings for now since this is not implemented yet

use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet;
use super::packet::{EntityLookAndMove, Packet, PlayerInfo, SpawnPlayer};
use std::sync::mpsc::Sender;
use uuid::Uuid;

// Called when requesting a peer subscription to another server
pub fn init_outgoing_peer_sub(p: Packet, conn_id: Uuid, messenger: Sender<MessengerOperations>) {
    send_packet!(
        messenger,
        conn_id,
        Packet::PlayerInfo(PlayerInfo {
            action: 0,
            number_of_players: 1, //send each player in an individual packet for now
            uuid: Uuid::new_v4().as_u128(),
            name: "fake man".to_string(),
            number_of_properties: 0,
            gamemode: 1,
            ping: 100,
            has_display_name: false,
        })
    )
    .unwrap();
}

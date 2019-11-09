use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet::{Packet, PlayerInfo, SpawnPlayer};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub enum PlayerStateOperations {
    New(NewPlayerMessage),
    Report(ReportMessage),
}

#[derive(Debug, Clone)]
pub struct Player {
    pub conn_id: u64,
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub struct NewPlayerMessage {
    pub conn_id: u64,
    pub player: Player,
}

#[derive(Debug)]
pub struct ReportMessage {
    pub conn_id: u64,
}

pub fn start_player_state(
    receiver: Receiver<PlayerStateOperations>,
    messenger: Sender<MessengerOperations>,
) {
    let mut players = HashMap::<u64, Player>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PlayerStateOperations::New(msg) => {
                players.insert(msg.conn_id, msg.player);
            }
            PlayerStateOperations::Report(msg) => {
                players.values().for_each(|player| {
                    let player_clone = player.clone();
                    send_packet!(
                        messenger,
                        msg.conn_id,
                        Packet::PlayerInfo(PlayerInfo {
                            action: 0,
                            number_of_players: 1, //send each player in an individual packet for now
                            uuid: player_clone.uuid.as_u128(),
                            name: player_clone.name.clone(),
                            number_of_properties: 0,
                            gamemode: 1,
                            ping: 100,
                            has_display_name: false,
                        })
                    )
                    .unwrap();
                    send_packet!(
                        messenger,
                        msg.conn_id,
                        Packet::SpawnPlayer(SpawnPlayer {
                            entity_id: player.conn_id,
                            uuid: player_clone.uuid.as_u128(),
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                            yaw: 0,
                            pitch: 0,
                            entity_metadata_terminator: 0xff,
                        })
                    )
                    .unwrap();
                })
            }
        }
    }
}

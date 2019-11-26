use super::messenger::{BroadcastPacketMessage, MessengerOperations, SendPacketMessage};
use super::packet::{EntityLookAndMove, Packet, PlayerInfo, SpawnPlayer};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub enum PlayerStateOperations {
    New(NewPlayerMessage),
    Report(ReportMessage),
    Move(PlayerMovementMessage),
}

#[derive(Debug, Clone)]
pub struct Player {
    pub conn_id: Uuid,
    pub uuid: Uuid,
    pub name: String,
    pub position: Position,
}

//This probably belongs at an entity level, but since we don't have a real concept of entities yet
//this'll do
//Ignoring angle since we haven't implemented that datatype just yet
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
pub struct PositionDelta {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl PositionDelta {
    pub fn new(old_position: Position, new_position: Position) -> PositionDelta {
        PositionDelta {
            x: ((new_position.x * 32.0 - old_position.x * 32.0) * 128.0) as i16,
            y: ((new_position.y * 32.0 - old_position.y * 32.0) * 128.0) as i16,
            z: ((new_position.z * 32.0 - old_position.z * 32.0) * 128.0) as i16,
        }
    }
}

#[derive(Debug)]
pub struct NewPlayerMessage {
    pub conn_id: Uuid,
    pub player: Player,
}

#[derive(Debug)]
pub struct ReportMessage {
    pub conn_id: Uuid,
}

#[derive(Debug)]
pub struct PlayerMovementMessage {
    pub conn_id: Uuid,
    pub new_position: Position,
}

pub fn start_player_state(
    receiver: Receiver<PlayerStateOperations>,
    messenger: Sender<MessengerOperations>,
) {
    let mut players = HashMap::<Uuid, Player>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PlayerStateOperations::New(msg) => {
                players.insert(msg.conn_id, msg.player);
            }
            PlayerStateOperations::Move(msg) => {
                let player = players.get(&msg.conn_id).unwrap();
                let mut player_clone = player.clone();
                let position_delta =
                    PositionDelta::new(player_clone.position, msg.new_position.clone());
                broadcast_packet!(
                    messenger,
                    Packet::EntityLookAndMove(EntityLookAndMove {
                        entity_id: player_clone.conn_id.as_u128() as u64,
                        delta_x: position_delta.x,
                        delta_y: position_delta.y,
                        delta_z: position_delta.z,
                        yaw: 0,
                        pitch: 0,
                        on_ground: false,
                    })
                )
                .unwrap();
                player_clone.position = msg.new_position;
                players.insert(msg.conn_id, player_clone);
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
                            entity_id: player.conn_id.as_u128() as u64,
                            uuid: player_clone.uuid.as_u128(),
                            x: player_clone.position.x,
                            y: player_clone.position.y,
                            z: player_clone.position.z,
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

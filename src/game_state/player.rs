use super::super::minecraft_protocol::float_to_angle;
use super::messenger::{BroadcastPacketMessage, MessengerOperations, SendPacketMessage};
use super::packet::{
    ClientboundPlayerPositionAndLook, EntityHeadLook, EntityLookAndMove, JoinGame, Packet,
    PlayerInfo, SpawnPlayer,
};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub enum PlayerStateOperations {
    New(NewPlayerMessage),
    Report(ReportMessage),
    MoveAndLook(PlayerMoveAndLookMessage),
}

#[derive(Debug, Clone)]
pub struct Player {
    pub conn_id: Uuid,
    pub uuid: Uuid,
    pub name: String,
    pub position: Position,
    pub angle: Angle,
    pub entity_id: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
pub struct Angle {
    pub pitch: f32,
    pub yaw: f32,
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
pub struct PlayerMoveAndLookMessage {
    pub conn_id: Uuid,
    pub new_position: Option<Position>,
    pub new_angle: Option<Angle>,
}

pub fn start(receiver: Receiver<PlayerStateOperations>, messenger: Sender<MessengerOperations>) {
    let mut players = HashMap::<Uuid, Player>::new();

    while let Ok(msg) = receiver.recv() {
        handle_message(msg, &mut players, messenger.clone())
    }
}

fn handle_message(
    msg: PlayerStateOperations,
    players: &mut HashMap<Uuid, Player>,
    messenger: Sender<MessengerOperations>,
) {
    match msg {
        PlayerStateOperations::New(msg) => {
            let mut player = msg.player;
            player.entity_id = players.len().try_into().expect("too many players");
            send_packet!(
                messenger,
                msg.conn_id,
                Packet::JoinGame(player.join_game_packet())
            )
            .unwrap();
            send_packet!(
                messenger,
                msg.conn_id,
                Packet::ClientboundPlayerPositionAndLook(player.pos_and_look_packet())
            )
            .unwrap();
            players.insert(msg.conn_id, player);
        }
        PlayerStateOperations::MoveAndLook(msg) => {
            players.entry(msg.conn_id).and_modify(|player| {
                broadcast_packet!(
                    messenger,
                    Packet::EntityLookAndMove(
                        player.move_and_look(msg.new_position, msg.new_angle)
                    ),
                    Some(player.conn_id),
                    true
                )
                .unwrap();
                broadcast_packet!(
                    messenger,
                    Packet::EntityHeadLook(player.entity_head_look()),
                    Some(player.conn_id),
                    true
                )
                .unwrap()
            });
        }
        PlayerStateOperations::Report(msg) => players.iter().for_each(|(conn_id, player)| {
            if *conn_id != msg.conn_id {
                send_packet!(
                    messenger,
                    msg.conn_id,
                    Packet::PlayerInfo(player.player_info_packet())
                )
                .unwrap();
                send_packet!(
                    messenger,
                    msg.conn_id,
                    Packet::SpawnPlayer(player.spawn_player_packet())
                )
                .unwrap();
            }
        }),
    }
}

impl Player {
    pub fn move_and_look(
        &mut self,
        new_position: Option<Position>,
        new_angle: Option<Angle>,
    ) -> EntityLookAndMove {
        if let Some(new_angle) = new_angle {
            self.angle = new_angle;
        }
        let update_packet = self.entity_look_and_move_packet(new_position);
        if let Some(new_position) = new_position {
            self.position = new_position;
        }
        update_packet
    }

    pub fn join_game_packet(&self) -> JoinGame {
        JoinGame {
            entity_id: self.entity_id,
            gamemode: 1,
            dimension: 0,
            difficulty: 0,
            max_players: 2,
            level_type: String::from("default"),
            reduced_debug_info: false,
        }
    }

    pub fn pos_and_look_packet(&self) -> ClientboundPlayerPositionAndLook {
        ClientboundPlayerPositionAndLook {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
            yaw: 0.0,
            pitch: 0.0,
            flags: 0,
            teleport_id: 0,
        }
    }

    fn entity_head_look(&self) -> EntityHeadLook {
        EntityHeadLook {
            entity_id: self.entity_id,
            angle: float_to_angle(self.angle.yaw),
        }
    }

    fn entity_look_and_move_packet(&self, new_position: Option<Position>) -> EntityLookAndMove {
        let position_delta = PositionDelta::new(self.position, new_position);
        EntityLookAndMove {
            entity_id: self.entity_id,
            delta_x: position_delta.x,
            delta_y: position_delta.y,
            delta_z: position_delta.z,
            yaw: float_to_angle(self.angle.yaw),
            pitch: float_to_angle(self.angle.pitch),
            on_ground: false,
        }
    }

    fn player_info_packet(&self) -> PlayerInfo {
        PlayerInfo {
            action: 0,
            number_of_players: 1, //send each player in an individual packet for now
            uuid: self.uuid.as_u128(),
            name: self.name.clone(),
            number_of_properties: 0,
            gamemode: 1,
            ping: 100,
            has_display_name: false,
        }
    }

    fn spawn_player_packet(&self) -> SpawnPlayer {
        SpawnPlayer {
            entity_id: self.entity_id,
            uuid: self.uuid.as_u128(),
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
            yaw: 0,
            pitch: 0,
            entity_metadata_terminator: 0xff,
        }
    }
}

#[derive(Debug, Clone)]
struct PositionDelta {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl PositionDelta {
    pub fn new(old_position: Position, new_position: Option<Position>) -> PositionDelta {
        match new_position {
            Some(position) => PositionDelta {
                x: ((position.x * 32.0 - old_position.x * 32.0) * 128.0) as i16,
                y: ((position.y * 32.0 - old_position.y * 32.0) * 128.0) as i16,
                z: ((position.z * 32.0 - old_position.z * 32.0) * 128.0) as i16,
            },
            None => PositionDelta { x: 0, y: 0, z: 0 },
        }
    }
}

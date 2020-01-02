use super::messenger::Messenger;
use super::minecraft_types::float_to_angle;
use super::packet::{
    ClientboundPlayerPositionAndLook, EntityHeadLook, EntityLookAndMove, JoinGame, Packet,
    PlayerInfo, PlayerPositionAndLook, SpawnPlayer,
};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::mpsc::Receiver;
use uuid::Uuid;

pub enum PlayerStateOperations {
    New(NewPlayerMessage),
    Report(ReportMessage),
    MoveAndLook(PlayerMoveAndLookMessage),
    CrossBorder(CrossBorderMessage),
    BroadcastAnchoredEvent(BroadcastAnchoredEventMessage),
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

#[derive(Debug, Clone)]
pub struct BroadcastAnchoredEventMessage {
    pub entity_id: i32,
    pub packet: Packet,
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
pub struct CrossBorderMessage {
    pub local_conn_id: Uuid,
    pub remote_conn_id: Uuid,
}

#[derive(Debug)]
pub struct PlayerMoveAndLookMessage {
    pub conn_id: Uuid,
    pub new_position: Option<Position>,
    pub new_angle: Option<Angle>,
}

pub fn start<M: Messenger + Clone>(receiver: Receiver<PlayerStateOperations>, messenger: M) {
    let mut players = HashMap::<Uuid, Player>::new();
    let mut entity_conn_ids = HashMap::<i32, Uuid>::new();

    while let Ok(msg) = receiver.recv() {
        handle_message(msg, &mut players, &mut entity_conn_ids, messenger.clone())
    }
}

fn handle_message<M: Messenger>(
    msg: PlayerStateOperations,
    players: &mut HashMap<Uuid, Player>,
    entity_conn_ids: &mut HashMap<i32, Uuid>,
    messenger: M,
) {
    match msg {
        PlayerStateOperations::New(msg) => {
            let mut player = msg.player;
            if player.entity_id == 0 {
                player.entity_id = players.len().try_into().expect("too many players");
            }
            trace!(
                "Creating new player {:?} for conn_id {:?}",
                player,
                msg.conn_id
            );
            messenger.send_packet(msg.conn_id, Packet::JoinGame(player.join_game_packet()));
            messenger.send_packet(
                msg.conn_id,
                Packet::ClientboundPlayerPositionAndLook(player.pos_and_look_packet()),
            );
            messenger.broadcast_packet(
                Packet::PlayerInfo(player.player_info_packet()),
                Some(msg.conn_id),
                true,
            );
            messenger.broadcast_packet(
                Packet::SpawnPlayer(player.spawn_player_packet()),
                Some(msg.conn_id),
                true,
            );
            entity_conn_ids.insert(player.entity_id, msg.conn_id);
            players.insert(msg.conn_id, player);
        }
        PlayerStateOperations::MoveAndLook(msg) => {
            trace!(
                "Player Move/Look new_position: {:?} new_angle: {:?} for conn_id {:?}",
                msg.new_position,
                msg.new_angle,
                msg.conn_id
            );
            players.entry(msg.conn_id).and_modify(|player| {
                messenger.broadcast_packet(
                    Packet::EntityLookAndMove(
                        player.move_and_look(msg.new_position, msg.new_angle),
                    ),
                    Some(player.conn_id),
                    true,
                );
                messenger.broadcast_packet(
                    Packet::EntityHeadLook(player.entity_head_look()),
                    Some(player.conn_id),
                    true,
                );
            });
        }
        PlayerStateOperations::Report(msg) => players.iter().for_each(|(conn_id, player)| {
            trace!("Reporting Player State to conn_id {:?}", conn_id);
            if *conn_id != msg.conn_id {
                messenger.send_packet(msg.conn_id, Packet::PlayerInfo(player.player_info_packet()));
                messenger.send_packet(
                    msg.conn_id,
                    Packet::SpawnPlayer(player.spawn_player_packet()),
                );
            }
        }),
        //When we get a message from a peer that comes from one of our anchored players we want to
        //make sure they don't get the result packets.
        PlayerStateOperations::BroadcastAnchoredEvent(msg) => {
            trace!("Broadcasting Anchored Event for entity {:?}", msg.entity_id);
            messenger.broadcast_packet(
                msg.packet,
                entity_conn_ids.get(&msg.entity_id).copied(),
                true,
            );
        }
        PlayerStateOperations::CrossBorder(msg) => {
            trace!("Crossing Border for conn_id {:?}", msg.local_conn_id);
            let player = players
                .get(&msg.local_conn_id)
                .expect("Could not cross border: player not found");
            messenger.send_packet(
                msg.remote_conn_id,
                Packet::PlayerPositionAndLook(player.player_position_and_look()),
            );
        }
    }
}

impl Player {
    pub fn player_position_and_look(&self) -> PlayerPositionAndLook {
        PlayerPositionAndLook {
            x: self.position.x,
            feet_y: self.position.y,
            z: self.position.z,
            yaw: self.angle.yaw,
            pitch: self.angle.pitch,
            on_ground: false,
        }
    }

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

use super::constants::SERVER_MAX_CAPACITY;
use super::interfaces::block::BlockState;
use super::interfaces::messenger::{Messenger, SubscriberType};
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::{Angle, Operations, Player, Position};
use super::minecraft_types;
use super::minecraft_types::float_to_angle;
use super::packet::{
    BorderCrossLogin, ClientboundPlayerPositionAndLook, DestroyEntities, EntityHeadLook,
    EntityLookAndMove, EntityTeleport, JoinGame, Packet, PlayerInfo, SpawnPlayer, StatusResponse,
};
use super::packet_handlers::initiation_protocols::login;
use std::collections::HashMap;

use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub fn start<M: Messenger + Clone, B: BlockState + Clone, PA: PatchworkState + Clone>(
    receiver: Receiver<Operations>,
    sender: Sender<Operations>,
    messenger: M,
    block_state: B,
    patchwork_state: PA,
) {
    let mut players = HashMap::<Uuid, Player>::new();
    let mut entity_conn_ids = HashMap::<i32, Uuid>::new();
    let mut entity_id = 0;

    while let Ok(msg) = receiver.recv() {
        handle_message(
            msg,
            &mut players,
            &mut entity_conn_ids,
            &mut entity_id,
            messenger.clone(),
            sender.clone(),
            block_state.clone(),
            patchwork_state.clone(),
        )
    }
}

fn handle_message<M: Messenger + Clone, B: BlockState + Clone, PA: PatchworkState + Clone>(
    msg: Operations,
    players: &mut HashMap<Uuid, Player>,
    entity_conn_ids: &mut HashMap<i32, Uuid>,
    entity_id: &mut i32,
    messenger: M,
    sender: Sender<Operations>,
    block_state: B,
    patchwork_state: PA,
) {
    match msg {
        Operations::New(msg) => {
            let mut player = msg.player;
            if player.entity_id == 0 {
                player.entity_id = *entity_id;
                *entity_id += 1;
            }
            trace!(
                "Creating new player {:?} for conn_id {:?}",
                player,
                msg.conn_id
            );
            messenger.broadcast(
                Packet::PlayerInfo(player.player_info_packet()),
                Some(msg.conn_id),
                SubscriberType::All,
            );
            messenger.broadcast(
                Packet::SpawnPlayer(player.spawn_player_packet()),
                Some(msg.conn_id),
                SubscriberType::All,
            );
            entity_conn_ids.insert(player.entity_id, msg.conn_id);
            players.insert(msg.conn_id, player);
        }
        Operations::Login(msg) => {
            if let Some(player) = players.get(&msg.conn_id) {
                login::initialize_world(
                    player.clone(),
                    messenger.clone(),
                    sender.clone(),
                    block_state.clone(),
                    patchwork_state.clone(),
                );
            }
        }
        Operations::Delete(msg) => {
            if let Some(player) = players.remove(&msg.conn_id) {
                messenger.broadcast(
                    Packet::DestroyEntities(DestroyEntities {
                        entity_ids: vec![player.entity_id],
                    }),
                    None,
                    SubscriberType::All,
                );
            }
        }
        Operations::MoveAndLook(msg) => {
            trace!(
                "Player Move/Look new_position: {:?} new_angle: {:?} for conn_id {:?}",
                msg.new_position,
                msg.new_angle,
                msg.conn_id
            );
            players.entry(msg.conn_id).and_modify(|player| {
                messenger.broadcast(
                    player.move_and_look(msg.new_position, msg.new_angle),
                    Some(player.conn_id),
                    SubscriberType::All,
                );
                messenger.broadcast(
                    Packet::EntityHeadLook(player.entity_head_look()),
                    Some(player.conn_id),
                    SubscriberType::All,
                );
            });
        }
        Operations::AnchoredMoveAndLook(msg) => {
            trace!(
                "Anchored Player Move/Look new_position: {:?} new_angle: {:?} for conn_id {:?}",
                msg.new_position,
                msg.new_angle,
                msg.conn_id
            );
            players.entry(msg.conn_id).and_modify(|player| {
                player.move_and_look(msg.new_position, msg.new_angle);
            });
        }
        Operations::Report(msg) => players.iter().for_each(|(conn_id, player)| {
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
        Operations::BroadcastAnchoredEvent(msg) => {
            if let Some(entity_id) = entity_conn_ids.get(&msg.entity_id) {
                trace!("Appending entity id {:?} to anchored event", entity_id);
            }
            messenger.broadcast(
                msg.packet,
                entity_conn_ids.get(&msg.entity_id).copied(),
                SubscriberType::Local,
            );
        }
        Operations::CrossBorder(msg) => {
            trace!("Crossing Border for conn_id {:?}", msg.local_conn_id);
            let player = players
                .get(&msg.local_conn_id)
                .expect("Could not cross border: player not found");
            messenger.broadcast(
                Packet::DestroyEntities(DestroyEntities {
                    entity_ids: vec![player.entity_id],
                }),
                None,
                SubscriberType::Remote,
            );
            messenger.send_packet(
                msg.remote_conn_id,
                Packet::BorderCrossLogin(player.border_cross_login()),
            );
        }
        Operations::Reintroduce(msg) => {
            trace!("Reintroducing player for conn_id {:?}", msg.conn_id);
            let player = players
                .get(&msg.conn_id)
                .expect("Could not reintroduce: player not found");
            messenger.broadcast(
                Packet::SpawnPlayer(player.spawn_player_packet()),
                None,
                SubscriberType::Remote,
            );
        }
        Operations::StatusResponse(msg) => {
            trace!(
                "Building and sending status ping response for conn_id {:?}",
                msg.conn_id
            );
            let status_response_object = minecraft_types::StatusResponse {
                version: msg.version,
                players: minecraft_types::PingPlayersInfo {
                    max: SERVER_MAX_CAPACITY,
                    online: players.len() as u16,
                    sample: players
                        .iter()
                        .map(|(id, player)| minecraft_types::PingSamplePlayer {
                            name: player.name.clone(),
                            id: id.to_string(),
                        })
                        .collect(),
                },
                description: msg.description,
            };
            let status_response = StatusResponse {
                json_response: serde_json::to_string(&status_response_object).unwrap(),
            };
            messenger.send_packet(msg.conn_id, Packet::StatusResponse(status_response));
        }
        Operations::TeleportToLastValidPos(msg) => {
            trace!(
                "Teleporting to last valid position for conn_id {:?}",
                msg.conn_id
            );
            let player = players
                .get(&msg.conn_id)
                .expect("Could not teleport to valid position: player not found");
            messenger.send_packet(
                msg.conn_id, 
                Packet::ClientboundPlayerPositionAndLook(player.pos_last_valid_packet(msg.look))
            );
        }
    }
}

impl Player {
    pub fn border_cross_login(&self) -> BorderCrossLogin {
        BorderCrossLogin {
            x: self.position.x,
            feet_y: self.position.y,
            z: self.position.z,
            yaw: self.angle.yaw,
            pitch: self.angle.pitch,
            on_ground: false,
            username: self.name.clone(),
            entity_id: self.entity_id,
        }
    }

    pub fn move_and_look(
        &mut self,
        new_position: Option<Position>,
        new_angle: Option<Angle>,
    ) -> Packet {
        if let Some(new_angle) = new_angle {
            self.angle = new_angle;
        }
        let update_packet = self.entity_displacement_packet(new_position);
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

    pub fn pos_last_valid_packet(&self, look: Option<Angle>) -> ClientboundPlayerPositionAndLook {
        ClientboundPlayerPositionAndLook {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
            yaw: look.unwrap_or(self.angle).yaw,
            pitch: look.unwrap_or(self.angle).pitch,
            flags: 0,
            teleport_id: 0
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

    fn entity_displacement_packet(&self, new_position: Option<Position>) -> Packet {
        let position_delta = PositionDelta::new(self.position, new_position);
        if position_distance(self.position, new_position) >= 8.0 {
            let new_position =
                new_position.expect("if we moved more than 8 blocks we must have moved");
            Packet::EntityTeleport(EntityTeleport {
                entity_id: self.entity_id,
                x: new_position.x,
                y: new_position.y,
                z: new_position.z,
                yaw: float_to_angle(self.angle.yaw),
                pitch: float_to_angle(self.angle.pitch),
                on_ground: false,
            })
        } else {
            Packet::EntityLookAndMove(EntityLookAndMove {
                entity_id: self.entity_id,
                delta_x: position_delta.x,
                delta_y: position_delta.y,
                delta_z: position_delta.z,
                yaw: float_to_angle(self.angle.yaw),
                pitch: float_to_angle(self.angle.pitch),
                on_ground: false,
            })
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

fn position_distance(old_position: Position, new_position: Option<Position>) -> f64 {
    match new_position {
        Some(position) => ((old_position.x - position.x).powf(2.0)
            + (old_position.y - position.y).powf(2.0)
            + (old_position.z - position.z).powf(2.0))
        .sqrt(),
        None => 0.0,
    }
}

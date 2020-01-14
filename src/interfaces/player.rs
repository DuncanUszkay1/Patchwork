use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub trait PlayerState {
    fn new_player(&self, conn_id: Uuid, player: Player);
    fn delete_player(&self, conn_id: Uuid);
    fn report(&self, conn_id: Uuid);
    fn move_and_look(
        &self,
        conn_id: Uuid,
        new_position: Option<Position>,
        new_angle: Option<Angle>,
    );
    fn cross_border(&self, local_conn_id: Uuid, remote_conn_id: Uuid);
    fn broadcast_anchored_event(&self, entity_id: i32, packet: Packet);
    fn reintroduce(&self, conn_id: Uuid);
}

impl PlayerState for Sender<PlayerStateOperations> {
    fn new_player(&self, conn_id: Uuid, player: Player) {
        self.send(PlayerStateOperations::New(NewPlayerMessage {
            conn_id,
            player,
        }))
        .unwrap();
    }
    fn delete_player(&self, conn_id: Uuid) {
        self.send(PlayerStateOperations::Delete(DeletePlayerMessage {
            conn_id,
        }))
        .unwrap();
    }
    fn report(&self, conn_id: Uuid) {
        self.send(PlayerStateOperations::Report(ReportMessage { conn_id }))
            .unwrap();
    }
    fn move_and_look(
        &self,
        conn_id: Uuid,
        new_position: Option<Position>,
        new_angle: Option<Angle>,
    ) {
        self.send(PlayerStateOperations::MoveAndLook(
            PlayerMoveAndLookMessage {
                conn_id,
                new_position,
                new_angle,
            },
        ))
        .unwrap();
    }
    fn cross_border(&self, local_conn_id: Uuid, remote_conn_id: Uuid) {
        self.send(PlayerStateOperations::CrossBorder(CrossBorderMessage {
            local_conn_id,
            remote_conn_id,
        }))
        .unwrap();
    }
    fn reintroduce(&self, conn_id: Uuid) {
        self.send(PlayerStateOperations::Reintroduce(ReintroduceMessage {
            conn_id,
        }))
        .unwrap();
    }
    fn broadcast_anchored_event(&self, entity_id: i32, packet: Packet) {
        self.send(PlayerStateOperations::BroadcastAnchoredEvent(
            BroadcastAnchoredEventMessage { entity_id, packet },
        ))
        .unwrap();
    }
}

pub enum PlayerStateOperations {
    New(NewPlayerMessage),
    Delete(DeletePlayerMessage),
    Report(ReportMessage),
    MoveAndLook(PlayerMoveAndLookMessage),
    CrossBorder(CrossBorderMessage),
    Reintroduce(ReintroduceMessage),
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
pub struct DeletePlayerMessage {
    pub conn_id: Uuid,
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
pub struct ReintroduceMessage {
    pub conn_id: Uuid,
}

#[derive(Debug)]
pub struct PlayerMoveAndLookMessage {
    pub conn_id: Uuid,
    pub new_position: Option<Position>,
    pub new_angle: Option<Angle>,
}

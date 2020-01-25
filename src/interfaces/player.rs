use super::minecraft_types::{Description, Version};
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(
    PlayerState,
    (Report, report, [conn_id: Uuid]),
    (New, new_player, [conn_id: Uuid, player: Player]),
    (Delete, delete_player, [conn_id: Uuid]),
    (
        MoveAndLook,
        move_and_look,
        [
            conn_id: Uuid,
            new_position: Option<Position>,
            new_angle: Option<Angle>
        ]
    ),
    (
        AnchoredMoveAndLook,
        anchored_move_and_look,
        [
            conn_id: Uuid,
            new_position: Option<Position>,
            new_angle: Option<Angle>
        ]
    ),
    (
        CrossBorder,
        cross_border,
        [local_conn_id: Uuid, remote_conn_id: Uuid]
    ),
    (
        BroadcastAnchoredEvent,
        broadcast_anchored_event,
        [entity_id: i32, packet: Packet]
    ),
    (Reintroduce, reintroduce, [conn_id: Uuid]),
    (
        StatusResponse, 
        status_response,
        [
            conn_id: Uuid, 
            version: Version, 
            description: Description
        ]
    )
);

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

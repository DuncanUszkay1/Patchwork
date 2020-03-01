use super::packet::{BlockChange, PlayerDigging, PlayerBlockPlacement};

use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(
    BlockState,
    (Report, report, [conn_id: Uuid]),
    (
        BlockPlacement,
        block_placement,
        [conn_id: Uuid, block_placement: PlayerBlockPlacement]
    ),
    (
        BreakBlock,
        break_block,
        [conn_id: Uuid, player_digging: PlayerDigging]
    )
);

#[derive(Debug, Clone)]
pub struct Block {
    pub conn_id: Uuid,
    pub block_ids: Vec::<Vec::<Vec::<Vec::<i32>>>>
}

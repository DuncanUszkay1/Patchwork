use super::packet::{BlockChange, PlayerDigging};

use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(
    BlockState,
    (Report, report, [conn_id: Uuid]),
    (
        BreakBlockClientbound,
        break_block_clientbound,
        [conn_id: Uuid, block_packet: BlockChange]
    ),
    (
        BreakBlockServerbound,
        break_block_serverbound,
        [conn_id: Uuid, block_packet: PlayerDigging]
    )
);

#[derive(Debug, Clone)]
pub struct Block {
    pub conn_id: Uuid,
    pub block_ids: Vec<i32>,
}

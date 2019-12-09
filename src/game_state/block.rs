use super::super::minecraft_protocol::ChunkSection;
use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet::{ChunkData, Packet};

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use uuid::Uuid;

// We don't really have any meaningful block state yet- it cannot be changed or be particularly
// complicated. We can build this up later

pub enum BlockStateOperations {
    Report(ReportMessage),
}

#[derive(Debug)]
pub struct ReportMessage {
    pub conn_id: Uuid,
}

fn fill_dummy_block_ids(ids: &mut Vec<i32>) {
    //just some random pattern
    while ids.len() < 4096 {
        match ids.len() % 2 {
            0 => {
                ids.push(5);
            }
            1 => {
                ids.push(3);
            }
            _ => {
                panic!("if this happens, all hope is lost");
            }
        }
    }
}

pub fn start(receiver: Receiver<BlockStateOperations>, messenger: Sender<MessengerOperations>) {
    while let Ok(msg) = receiver.recv() {
        match msg {
            BlockStateOperations::Report(msg) => {
                //Just send a hardcoded simple chunk pillar
                let mut block_ids = Vec::new();
                fill_dummy_block_ids(&mut block_ids);
                send_packet!(
                    messenger,
                    msg.conn_id,
                    Packet::ChunkData(ChunkData {
                        chunk_x: 0,
                        chunk_z: 0,
                        full_chunk: true,
                        primary_bit_mask: 1,
                        size: 12291, //I just calculated the length of this hardcoded chunk section
                        data: ChunkSection {
                            bits_per_block: 14,
                            data_array_length: 896,
                            block_ids,
                            block_light: Vec::new(),
                            sky_light: Vec::new(),
                        },
                        biomes: vec![127; 256],
                        number_of_block_entities: 0,
                    })
                )
                .unwrap();
            }
        }
    }
}

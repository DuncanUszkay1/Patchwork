use super::interfaces::block::Block;
use super::interfaces::block::Operations;
use super::interfaces::messenger::Messenger;
use super::minecraft_types::{BlockPosition, ChunkSection};
use super::packet::{BlockChange, ChunkData, Packet, PlayerDigging};

use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

// We don't really have any meaningful block state yet- it cannot be changed or be particularly
// complicated. We can build this up later
fn fill_dummy_block_ids(ids: &mut Vec<i32>) {
    while ids.len() < 4096 {
        let xz_pos = ids.len() % 256;
        let z_pos = xz_pos / 16;
        let x_pos = xz_pos % 16;
        if x_pos == 0 || x_pos == 15 || z_pos == 0 || z_pos == 15 {
            ids.push(180)
        } else {
            match (x_pos + z_pos) % 2 {
                0 => ids.push(97),
                1 => ids.push(103),
                _ => panic!("math has failed us."),
            }
        }
    }
}

pub fn start<M: Messenger + Clone>(
    receiver: Receiver<Operations>,
    _sender: Sender<Operations>,
    messenger: M,
) {
    let mut block = Block::new();
    while let Ok(msg) = receiver.recv() {
        //Just send a hardcoded simple chunk pillar
        println!("Block state started");
        handle_message(msg, &mut block, messenger.clone());
    }
}

fn handle_message<M: Messenger + Clone>(msg: Operations, block: &mut Block, messenger: M) {
    match msg {
        Operations::Report(msg) => {
            trace!("Reporting block state to {:?}", msg.conn_id);
            println!("Reporting block state to {:?}", msg.conn_id);

            refresh_chunk(msg.conn_id, &mut block.block_ids, messenger);
        }
        Operations::BreakBlockClientbound(msg) => {
            println!("Block broken (clientbound)");
            block.break_block_clientbound(BlockChange {
                location: msg.block_packet.location,
                block_id: 0,
            });
            refresh_chunk(msg.conn_id, &mut block.block_ids, messenger);
        }
        Operations::BreakBlockServerbound(msg) => {
            println!("Block broken (serverbound)");
            block.break_block_serverbound(PlayerDigging {
                status: 2,
                location: msg.block_packet.location,
                face: 0,
            });
            refresh_chunk(msg.conn_id, &mut block.block_ids, messenger);
        }
    }
}

impl Block {
    pub fn new() -> Block {
        let mut block_ids = Vec::new();
        fill_dummy_block_ids(&mut block_ids);
        Block {
            conn_id: Uuid::new_v4(),
            block_ids,
        }
    }
    pub fn break_block_clientbound(&mut self, block_packet: BlockChange) {
        println!("block broken (clientbound, Block method)");
        let i = get_1d_index(block_packet.location);
        self.block_ids[i] = 0;
    }

    pub fn break_block_serverbound(&mut self, block_packet: PlayerDigging) {
        println!("block broken (serverbound, Block method)");
        let i = get_1d_index(block_packet.location);
        self.block_ids[i] = 0;
    }
}

fn get_1d_index(pos: BlockPosition) -> usize {
    let mut i: usize = 0;

    let mut x1d: u32 = 0;
    let mut y1d: u16 = 0;
    let mut z1d: u32 = 0;
    while (pos.x > x1d && pos.y > y1d && pos.z > z1d) || i < 4095 {
        y1d += (i % 256) as u16;
        z1d += u32::from(y1d / 16);
        x1d += u32::from(y1d % 16);

        i += 1;
    }
    i
}

fn refresh_chunk<M: Messenger + Clone>(conn_id: Uuid, block_ids: &mut Vec<i32>, messenger: M) {
    messenger.send_packet(
        conn_id,
        Packet::ChunkData(ChunkData {
            chunk_x: 0,
            chunk_z: 0,
            full_chunk: true,
            primary_bit_mask: 1,
            size: 12291, //I just calculated the length of this hardcoded chunk section
            data: ChunkSection {
                bits_per_block: 14,
                data_array_length: 896,
                block_ids: block_ids.to_vec(),
                block_light: Vec::new(),
                sky_light: Vec::new(),
            },
            biomes: vec![127; 256],
            number_of_block_entities: 0,
        }),
    );
}

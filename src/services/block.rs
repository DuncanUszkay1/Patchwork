use super::interfaces::block::Block;
use super::interfaces::block::Operations;
use super::interfaces::messenger::{Messenger, SubscriberType};
use super::minecraft_types::{BlockPosition, ChunkSection};
use super::packet::{BlockChange, ChunkData, Packet, PlayerDigging, PlayerBlockPlacement};

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
        handle_message(msg, &mut block, messenger.clone());
    }
}

fn handle_message<M: Messenger + Clone>(msg: Operations, block: &mut Block, messenger: M) {
    match msg {
        Operations::Report(msg) => {
            trace!("Reporting block state to {:?}", msg.conn_id);

            refresh_chunk(msg.conn_id, &mut block.block_ids[0][0][2], messenger.clone());
        }
        Operations::BlockPlacement(msg) => {
            trace!("Block placed: #{:?}", msg);
            let placement_position = get_position_of_placement(
              msg.block_placement.location,
              Face::from_varint(msg.block_placement.face)
            );
            block.place_block(placement_position);
            messenger.broadcast(Packet::BlockChange(BlockChange {
                location: placement_position,
                block_id: 1
            }), None, SubscriberType::All);
        }
        Operations::BreakBlock(msg) => {
            println!("break block {:?}", msg);
            block.break_block(msg.player_digging.location);
            messenger.broadcast(Packet::BlockChange(BlockChange {
                location: msg.player_digging.location,
                block_id: 0
            }), None, SubscriberType::All);
        }
    }
}

impl Block {
    pub fn new() -> Block {
        let mut block_ids = Vec::<Vec::<Vec::<Vec::<i32>>>>::new();
        let mut starting_pillar_row = Vec::new();
        let mut starting_pillar = Vec::new();
        let mut starting_chunk = Vec::new();
        let mut air_chunk = Vec::new();
        for i in 0..4096 { air_chunk.push(0); }
        fill_dummy_block_ids(&mut starting_chunk);
        for i in 0..2 { starting_pillar.push(air_chunk.clone()) }
        starting_pillar.push(starting_chunk);
        for i in 3..16 { starting_pillar.push(air_chunk.clone()) }
        starting_pillar_row.push(starting_pillar);
        block_ids.push(starting_pillar_row);
        Block {
            conn_id: Uuid::new_v4(),
            block_ids,
        }
    }
    pub fn place_block(&mut self, position: BlockPosition) {
        println!("place at {:?} {:?} {:?} {:?}", get_pillar_row_index(position),
            get_pillar_index(position),
            get_section_index(position),
            get_block_index(position)
            );
        self.block_ids
            [get_pillar_row_index(position)]
            [get_pillar_index(position)]
            [get_section_index(position)]
            [get_block_index(position)] = 1;
    }
    pub fn break_block(&mut self, position: BlockPosition) {
        self.block_ids
            [get_pillar_row_index(position)]
            [get_pillar_index(position)]
            [get_section_index(position)]
            [get_block_index(position)] = 0;
    }
}

fn get_pillar_row_index(pos: BlockPosition) -> usize {
    (pos.x / 16) as usize
}

fn get_pillar_index(pos: BlockPosition) -> usize {
    (pos.z / 16) as usize
}

fn get_section_index(pos: BlockPosition) -> usize {
    (pos.y / 16) as usize
}

fn get_block_index(pos: BlockPosition) -> usize {
    ((pos.x % 16) + (pos.z % 16) * 16 + (pos.y % 16) * 256) as usize
}

#[derive(Debug)]
enum Face {
    Bottom,
    Top,
    North,
    South,
    West,
    East
}

impl Face {
    pub fn from_varint(x: i32) -> Face {
        match x {
            0 => Face::Bottom,
            1 => Face::Top,
            2 => Face::North,
            3 => Face::South,
            4 => Face::West,
            5 => Face::East,
            _ => panic!("Received invalid face")
        }
    }
}

fn get_position_of_placement(position: BlockPosition, face: Face) -> BlockPosition {
    let mut adjusted_position = position;
    match face {
        Face::Bottom => adjusted_position.y -= 1,
        Face::Top => adjusted_position.y += 1,
        Face::North => adjusted_position.z -= 1,
        Face::South => adjusted_position.z += 1,
        Face::West => adjusted_position.x -= 1,
        Face::East => adjusted_position.x += 1
    }
    adjusted_position
}

fn refresh_chunk<M: Messenger + Clone>(conn_id: Uuid, block_ids: &mut Vec<i32>, messenger: M) {
    println!("block ids {:?}", block_ids);
    messenger.send_packet(
        conn_id,
        Packet::ChunkData(ChunkData {
            chunk_x: 0,
            chunk_z: 0,
            full_chunk: true,
            primary_bit_mask: 0b00000100,
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

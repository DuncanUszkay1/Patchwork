extern crate byteorder;

use byteorder::ReadBytesExt;
use std::io::{Error, Read};

pub fn float_to_angle(f: f32) -> u8 {
    ((f / 360.0) * 256.0) as u8
}

pub fn read_var_int<S: Read>(stream: &mut S) -> Result<i32, Error> {
    let mut num_read = 0;
    let mut result: i32 = 0;

    loop {
        let value = i32::from(stream.read_u8()?);
        result |= (value & 0b0111_1111) << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            panic!("VarInt is too big");
        }
        if (value & 0b1000_0000) == 0 {
            break;
        }
    }

    Ok(result)
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub bits_per_block: u8, //always 14 until we implement palettes
    pub data_array_length: i32,
    pub block_ids: Vec<i32>,   //4096 block ids
    pub block_light: Vec<u64>, //2048 bytes (all 1s)
    pub sky_light: Vec<u64>,   //2048 bytes (all 1s)
}

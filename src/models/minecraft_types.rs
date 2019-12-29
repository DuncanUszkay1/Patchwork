pub fn float_to_angle(f: f32) -> u8 {
    ((f / 360.0) * 256.0) as u8
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub bits_per_block: u8, //always 14 until we implement palettes
    pub data_array_length: i32,
    pub block_ids: Vec<i32>,   //4096 block ids
    pub block_light: Vec<u64>, //2048 bytes (all 1s)
    pub sky_light: Vec<u64>,   //2048 bytes (all 1s)
}

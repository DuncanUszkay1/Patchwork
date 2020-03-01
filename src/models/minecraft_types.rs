use serde::{Deserialize, Serialize};

pub fn float_to_angle(f: f32) -> u8 {
    ((f / 360.0) * 256.0) as u8
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkSection {
    pub bits_per_block: u8, //always 14 until we implement palettes
    pub data_array_length: i32,
    pub block_ids: Vec<i32>,   //4096 block ids
    pub block_light: Vec<u64>, //2048 bytes (all 1s)
    pub sky_light: Vec<u64>,   //2048 bytes (all 1s)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub protocol: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingSamplePlayer {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingPlayersInfo {
    pub max: u16,
    pub online: u16,
    pub sample: Vec<PingSamplePlayer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Description {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: Version,
    pub players: PingPlayersInfo,
    pub description: Description,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockPosition {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#![allow(unused_variables)]
//Had to disable unused variables here since it wasn't working with packet::new
use super::minecraft_protocol::{
    write_var_int, ChunkSection, MinecraftProtocolReader, MinecraftProtocolWriter,
};
use std::io::{Cursor, Read, Write};

// Format: (state (99 is outgoing), name, id, [ list of (field name, field type) ]
packet_boilerplate!(
    (
        0,
        Handshake,
        0,
        [
            (protocol_version, VarInt),
            (server_address, String),
            (server_port, UShort),
            (next_state, VarInt)
        ]
    ),
    (1, StatusRequest, 0, []),
    (1, Ping, 1, [(payload, Long)]),
    (2, LoginStart, 0, [(username, String)]),
    (3, KeepAlive, 0x21, [(id, Long)]),
    (99, Pong, 1, [(payload, Long)]),
    (99, StatusResponse, 0, [(json_response, String)]),
    (99, LoginSuccess, 2, [(uuid, String), (username, String)]),
    (
        99,
        JoinGame,
        0x25,
        [
            (entity_id, Int),
            (gamemode, UByte),
            (dimension, Int),
            (difficulty, UByte),
            (max_players, UByte),
            (level_type, String),
            (reduced_debug_info, Boolean)
        ]
    ),
    (
        99,
        PlayerPositionAndLook,
        0x32,
        [
            (x, Double),
            (y, Double),
            (z, Double),
            (yaw, Float),
            (pitch, Float),
            (flags, Byte),
            (teleport_id, VarInt)
        ]
    ),
    (
        99,
        ChunkData,
        0x22,
        [
            (chunk_x, Int),
            (chunk_z, Int),
            (full_chunk, Boolean), //always true
            (primary_bit_mask, VarInt),
            (size, VarInt),
            (data, ChunkSection), //actually a chunk array, but can pretend its 1 for now
            (biomes, IntArray),
            (number_of_block_entities, VarInt)
        ]
    ),
    (
        99,
        PlayerInfo,
        0x30,
        [
            (action, VarInt), //always 0
            (number_of_players, VarInt),
            (uuid, u128),
            (name, String),
            (number_of_properties, VarInt), //always 0
            (gamemode, VarInt),
            (ping, VarInt),
            (has_display_name, Boolean) //always false
        ]
    ),
    (
        99,
        SpawnPlayer,
        0x05,
        [
            (entity_id, VarInt),
            (uuid, u128),
            (x, Double),
            (y, Double),
            (z, Double),
            (yaw, UByte), // represents angle * (360/256). Might want to eventually make this its own type
            (pitch, UByte), // for now lets just set it to 0
            (entity_metadata_terminator, UByte)  // always 0xff until we implement entity metadata
        ]
    )
);

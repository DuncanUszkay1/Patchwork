#![allow(unused_variables)]
//Had to disable unused variables here since it wasn't working with packet::new
use super::game_state::patchwork::{CHUNK_SIZE, ENTITY_ID_BLOCK_SIZE};
use super::minecraft_protocol::{
    write_var_int, ChunkSection, MinecraftProtocolReader, MinecraftProtocolWriter,
};
use super::packet_processor::TranslationInfo;
use std::io::{Cursor, Read, Write};

// Format: (state (99 is outgoing), name, id, [ list of (field name, field type) ]
#[rustfmt::skip::macros(packet_boilerplate)]
packet_boilerplate!(
    (
        0,
        Handshake,
        0,
        [
            (protocol_version, VarInt, Untranslated),
            (server_address, String, Untranslated),
            (server_port, UShort, Untranslated),
            (next_state, VarInt, Untranslated)
        ]
    ),
    (1, StatusRequest, 0, []),
    (1, Ping, 1, [(payload, Long, Untranslated)]),
    (2, LoginStart, 0, [(username, String, Untranslated)]),
    (3, KeepAlive, 0x21, [(id, Long, Untranslated)]),
    (
        3,
        PlayerPosition,
        0x10,
        [
            (x, Double, Untranslated),
            (feet_y, Double, Untranslated),
            (z, Double, Untranslated),
            (on_ground, Boolean, Untranslated)
        ]
    ),
    (
        3,
        PlayerPositionAndLook,
        0x11,
        [
            (x, Double, Untranslated),
            (feet_y, Double, Untranslated),
            (z, Double, Untranslated),
            (yaw, Float, Untranslated),
            (pitch, Float, Untranslated),
            (on_ground, Boolean, Untranslated)
        ]
    ),
    (
        3,
        PlayerLook,
        0x12,
        [
            (yaw, Float, Untranslated),
            (pitch, Float, Untranslated),
            (on_ground, Boolean, Untranslated)
        ]
    ),
    (6, ReportState, 0x1, []),
    (99, Pong, 1, [(payload, Long, Untranslated)]),
    (99, StatusResponse, 0, [(json_response, String, Untranslated)]),
    (99, LoginSuccess, 2, [(uuid, String, Untranslated), (username, String, Untranslated)]),
    (
        99,
        JoinGame,
        0x25,
        [
            (entity_id, Int, EntityId),
            (gamemode, UByte, Untranslated),
            (dimension, Int, Untranslated),
            (difficulty, UByte, Untranslated),
            (max_players, UByte, Untranslated),
            (level_type, String, Untranslated),
            (reduced_debug_info, Boolean, Untranslated)
        ]
    ),
    (
        99,
        ClientboundPlayerPositionAndLook,
        0x32,
        [
            (x, Double, Untranslated),
            (y, Double, Untranslated),
            (z, Double, Untranslated),
            (yaw, Float, Untranslated),
            (pitch, Float, Untranslated),
            (flags, Byte, Untranslated),
            (teleport_id, VarInt, Untranslated)
        ]
    ),
    (
        _,
        ChunkData,
        0x22,
        [
            (chunk_x, Int, XChunk),
            (chunk_z, Int, Untranslated),
            (full_chunk, Boolean, Untranslated), //always true
            (primary_bit_mask, VarInt, Untranslated),
            (size, VarInt, Untranslated),
            (data, ChunkSection, Untranslated), //actually a chunk array, but can pretend its 1 for now
            (biomes, IntArray, Untranslated),
            (number_of_block_entities, VarInt, Untranslated)
        ]
    ),
    (
        _,
        PlayerInfo,
        0x30,
        [
            (action, VarInt, Untranslated),
            (number_of_players, VarInt, Untranslated),
            (uuid, u128, Untranslated),
            (name, String, Untranslated),
            (number_of_properties, VarInt, Untranslated),
            (gamemode, VarInt, Untranslated),
            (ping, VarInt, Untranslated),
            (has_display_name, Boolean, Untranslated)
        ]
    ),
    (
        _,
        SpawnPlayer,
        0x05,
        [
            (entity_id, VarInt, EntityId),
            (uuid, u128, Untranslated),
            (x, Double, XEntity),
            (y, Double, Untranslated),
            (z, Double, Untranslated),
            (yaw, UByte, Untranslated), // represents angle * (360/256). Might want to eventually make this its own type
            (pitch, UByte, Untranslated), // for now lets just set it to 0
            (entity_metadata_terminator, UByte, Untranslated)  // always 0xff until we implement entity metadata
        ]
    ),
    (
        _,
        EntityLookAndMove,
        0x29,
        [
            (entity_id, VarInt, EntityId),
            (delta_x, Short, Untranslated),
            (delta_y, Short, Untranslated),
            (delta_z, Short, Untranslated),
            (yaw, UByte, Untranslated),
            (pitch, UByte, Untranslated),
            (on_ground, Boolean, Untranslated)
        ]
    )
);

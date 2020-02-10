use super::interfaces::block::BlockState;
use super::interfaces::messenger::{Messenger, SubscriberType};
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::{Angle, Player, PlayerState, Position};
use super::interfaces::report::report;
use super::minecraft_types::ChunkSection;
use super::packet;
use super::packet::{ChunkData, Packet};
use super::translation::TranslationUpdates;
use rand::prelude::*;
use uuid::Uuid;

pub fn handle_login_packet<M: Messenger + Clone, P: PlayerState + Clone>(
    p: Packet,
    conn_id: Uuid,
    messenger: M,
    player_state: P,
) -> TranslationUpdates {
    match p {
        Packet::LoginStart(login_start) => {
            confirm_login(conn_id, messenger, login_start, player_state);
            TranslationUpdates::State(3)
        }
        _ => {
            panic!("Login failed");
        }
    }
}

pub fn initialize_world<M: Messenger + Clone, B: BlockState, PA: PatchworkState, P: PlayerState>(
    player: Player,
    messenger: M,
    player_state: P,
    block_state: B,
    patchwork_state: PA,
) {
    messenger.send_packet(player.conn_id, Packet::JoinGame(player.join_game_packet()));
    send_placeholder_chunks(player.conn_id, messenger.clone());
    messenger.send_packet(
        player.conn_id,
        Packet::ClientboundPlayerPositionAndLook(player.pos_and_look_packet()),
    );
    report(
        player.conn_id,
        messenger,
        player_state,
        block_state,
        patchwork_state,
        SubscriberType::Local,
    );
}

fn send_placeholder_chunks<M: Messenger + Clone>(conn_id: Uuid, messenger: M) {
    for x in -20..20 {
        for z in -20..20 {
            send_placeholder_chunk(x, z, conn_id, messenger.clone());
        }
    }
}

fn send_placeholder_chunk<M: Messenger>(x: i32, z: i32, conn_id: Uuid, messenger: M) {
    let mut block_ids = Vec::new();
    let mut pillar_heights = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..256 {
        pillar_heights.push(rng.gen_range(1, 10));
        block_ids.push(108)
    }
    for y in 1..16 {
        for z in 0..16 {
            for x in 0..16 {
                if pillar_heights[x + 16 * z] > y {
                    block_ids.push(180)
                } else {
                    block_ids.push(0)
                }
            }
        }
    }
    messenger.send_packet(
        conn_id,
        Packet::ChunkData(ChunkData {
            chunk_x: x,
            chunk_z: z,
            full_chunk: true,
            primary_bit_mask: 2_i32.pow(3),
            size: 12291,
            data: ChunkSection {
                bits_per_block: 14,
                data_array_length: 896,
                block_ids,
                block_light: Vec::new(),
                sky_light: Vec::new(),
            },
            biomes: vec![127; 256],
            number_of_block_entities: 0,
        }),
    );
}

fn confirm_login<M: Messenger + Clone, P: PlayerState + Clone>(
    conn_id: Uuid,
    messenger: M,
    login_start: packet::LoginStart,
    player_state: P,
) {
    let player = Player {
        conn_id,
        uuid: Uuid::new_v4(),
        name: login_start.username,
        entity_id: 0, // replaced by player state
        position: Position {
            x: 10.0,
            y: 80.0,
            z: 10.0,
        },
        angle: Angle {
            pitch: 0.0,
            yaw: 0.0,
        },
    };

    //protocol
    login_success(conn_id, messenger.clone(), player.clone());
    player_state.new_player(conn_id, player);
    player_state.login(conn_id);
    messenger.subscribe(conn_id, SubscriberType::All);
}

fn login_success<M: Messenger>(conn_id: Uuid, messenger: M, player: Player) {
    let login_success = packet::LoginSuccess {
        uuid: player.uuid.to_hyphenated().to_string(),
        username: player.name,
    };
    messenger.send_packet(conn_id, Packet::LoginSuccess(login_success));
}

use super::super::minecraft_protocol::ChunkSection;
use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet;
use super::packet::Packet;
use std::sync::mpsc::Sender;

// Called upon user login
pub fn init_login(
    p: Packet,
    state: &mut u64,
    conn_id: i32,
    messenger: Sender<MessengerOperations>,
) {
    println!("Login protocol initiated :{:?}", p);
    match p.clone() {
        Packet::LoginStart(login_start) => {
            *state = 3;
            confirm_login(conn_id, messenger, login_start);
        }
        _ => {
            println!("Login failed");
        }
    }
}

fn confirm_login(
    conn_id: i32,
    messenger: Sender<MessengerOperations>,
    login_start: packet::LoginStart,
) {
    println!("Joining ...");
    let login_success = packet::LoginSuccess {
        uuid: "1c88a2d0-cdf1-4999-9dc1-3e2f696dde05".to_string(),
        username: login_start.username,
    };
    println!("{:?}", login_success);
    send_packet!(messenger, conn_id, Packet::LoginSuccess(login_success)).unwrap();
    println!("Joining the game ...");
    let join_game = packet::JoinGame {
        entity_id: 0,
        gamemode: 1,
        dimension: 0,
        difficulty: 0,
        max_players: 2,
        level_type: "default".to_string(),
        reduced_debug_info: false,
    };
    println!("{:?}", join_game);
    send_packet!(messenger, conn_id, Packet::JoinGame(join_game)).unwrap();
    println!("Seeting player's position and camera ...");
    let player_pos_and_look = packet::PlayerPositionAndLook {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        flags: 0,
        teleport_id: 0,
    };
    println!("{:?}", player_pos_and_look);
    send_packet!(
        messenger,
        conn_id,
        Packet::PlayerPositionAndLook(player_pos_and_look)
    )
    .unwrap();
    let chunk_data = packet::ChunkData {
        chunk_x: 0,
        chunk_z: 0,
        full_chunk: true,
        primary_bit_mask: 1,
        size: 12291, //I just calculated the length of this hardcoded chunk section
        data: ChunkSection {
            bits_per_block: 14,
            data_array_length: 896,
            block_ids: Vec::new(),
            block_light: Vec::new(),
            sky_light: Vec::new(),
        },
        biomes: vec![127; 256],
        number_of_block_entities: 0,
    };
    send_packet!(messenger, conn_id, Packet::ChunkData(chunk_data)).unwrap();
}

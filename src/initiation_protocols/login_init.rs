use super::super::minecraft_protocol::ChunkSection;
use super::game_state::patchwork::PatchworkStateOperations;
use super::game_state::player;
use super::game_state::player::{NewPlayerMessage, Player, PlayerStateOperations, Position};
use super::messenger::{MessengerOperations, SendPacketMessage, SubscribeMessage};
use super::packet;
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

// Called upon user login
pub fn init_login(
    p: Packet,
    conn_id: Uuid,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
    patchwork_state: Sender<PatchworkStateOperations>,
) {
    println!("Login protocol initiated :{:?}", p);
    match p.clone() {
        Packet::LoginStart(login_start) => {
            confirm_login(
                conn_id,
                messenger,
                login_start,
                player_state,
                patchwork_state,
            );
        }
        _ => {
            println!("Login failed");
        }
    }
}

fn confirm_login(
    conn_id: Uuid,
    messenger: Sender<MessengerOperations>,
    login_start: packet::LoginStart,
    player_state: Sender<PlayerStateOperations>,
    patchwork_state: Sender<PatchworkStateOperations>,
) {
    let player = Player {
        conn_id,
        uuid: Uuid::new_v4(),
        name: login_start.username,
        entity_id: 0, // replaced by player state
        position: Position {
            x: 5.0,
            y: 16.0,
            z: 5.0,
        },
    };

    //protocol
    login_success(conn_id, messenger.clone(), player.clone());
    join_game(conn_id, messenger.clone());
    set_position(conn_id, messenger.clone(), player.clone());
    temp_send_block_data(conn_id, messenger.clone());

    messenger
        .send(MessengerOperations::Subscribe(SubscribeMessage { conn_id }))
        .unwrap();

    //report current state to player (soon to be in it's own component for reuse)
    //the only state we keep right now is players
    player_state
        .send(PlayerStateOperations::Report(player::ReportMessage {
            conn_id,
        }))
        .unwrap();

    patchwork_state
        .send(PatchworkStateOperations::Report)
        .unwrap();

    //update the gamestate with this new player
    player_state
        .send(PlayerStateOperations::New(NewPlayerMessage {
            conn_id,
            player,
        }))
        .unwrap();
}

fn login_success(conn_id: Uuid, messenger: Sender<MessengerOperations>, player: Player) {
    let login_success = packet::LoginSuccess {
        uuid: player.uuid.to_hyphenated().to_string(),
        username: player.name,
    };
    send_packet!(messenger, conn_id, Packet::LoginSuccess(login_success)).unwrap();
}

fn join_game(conn_id: Uuid, messenger: Sender<MessengerOperations>) {
    let join_game = packet::JoinGame {
        entity_id: 0,
        gamemode: 1,
        dimension: 0,
        difficulty: 0,
        max_players: 2,
        level_type: String::from("default"),
        reduced_debug_info: false,
    };
    send_packet!(messenger, conn_id, Packet::JoinGame(join_game)).unwrap();
}

fn set_position(conn_id: Uuid, messenger: Sender<MessengerOperations>, player: Player) {
    let player_pos_and_look = packet::ClientboundPlayerPositionAndLook {
        x: player.position.x,
        y: player.position.y,
        z: player.position.z,
        yaw: 0.0,
        pitch: 0.0,
        flags: 0,
        teleport_id: 0,
    };
    send_packet!(
        messenger,
        conn_id,
        Packet::ClientboundPlayerPositionAndLook(player_pos_and_look)
    )
    .unwrap();
}

fn temp_send_block_data(conn_id: Uuid, messenger: Sender<MessengerOperations>) {
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

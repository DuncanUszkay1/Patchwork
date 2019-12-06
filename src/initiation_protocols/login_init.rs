use super::game_state::block;
use super::game_state::block::BlockStateOperations;
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
    block_state: Sender<BlockStateOperations>,
    patchwork_state: Sender<PatchworkStateOperations>,
) {
    match p.clone() {
        Packet::LoginStart(login_start) => {
            confirm_login(
                conn_id,
                messenger,
                login_start,
                player_state,
                block_state,
                patchwork_state,
            );
        }
        _ => {
            panic!("Login failed");
        }
    }
}

fn confirm_login(
    conn_id: Uuid,
    messenger: Sender<MessengerOperations>,
    login_start: packet::LoginStart,
    player_state: Sender<PlayerStateOperations>,
    block_state: Sender<BlockStateOperations>,
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

    //update the gamestate with this new player
    player_state
        .send(PlayerStateOperations::New(NewPlayerMessage {
            conn_id,
            player,
        }))
        .unwrap();

    block_state
        .send(BlockStateOperations::Report(block::ReportMessage {
            conn_id,
        }))
        .unwrap();

    messenger
        .send(MessengerOperations::Subscribe(SubscribeMessage {
            conn_id,
            local: true,
        }))
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
}

fn login_success(conn_id: Uuid, messenger: Sender<MessengerOperations>, player: Player) {
    let login_success = packet::LoginSuccess {
        uuid: player.uuid.to_hyphenated().to_string(),
        username: player.name,
    };
    send_packet!(messenger, conn_id, Packet::LoginSuccess(login_success)).unwrap();
}

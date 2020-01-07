use super::interfaces::block::BlockState;
use super::interfaces::messenger::{Messenger, SubscriberType};
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::{Angle, Player, PlayerState, Position};
use super::packet;
use super::packet::Packet;
use super::translation::TranslationUpdates;
use uuid::Uuid;

pub fn handle_login_packet<
    M: Messenger + Clone,
    P: PlayerState + Clone,
    PA: PatchworkState + Clone,
    B: BlockState + Clone,
>(
    p: Packet,
    conn_id: Uuid,
    messenger: M,
    player_state: P,
    block_state: B,
    patchwork_state: PA,
) -> TranslationUpdates {
    match p {
        Packet::LoginStart(login_start) => {
            confirm_login(
                conn_id,
                messenger,
                login_start,
                player_state,
                block_state,
                patchwork_state,
            );
            TranslationUpdates::State(3)
        }
        _ => {
            panic!("Login failed");
        }
    }
}

fn confirm_login<
    M: Messenger + Clone,
    P: PlayerState + Clone,
    PA: PatchworkState + Clone,
    B: BlockState + Clone,
>(
    conn_id: Uuid,
    messenger: M,
    login_start: packet::LoginStart,
    player_state: P,
    block_state: B,
    patchwork_state: PA,
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
        angle: Angle {
            pitch: 0.0,
            yaw: 0.0,
        },
    };

    //protocol
    login_success(conn_id, messenger.clone(), player.clone());

    //update the gamestate with this new player
    player_state.new_player(conn_id, player);
    block_state.report(conn_id);
    messenger.subscribe(conn_id, SubscriberType::All);
    player_state.report(conn_id);
    patchwork_state.report();
}

fn login_success<M: Messenger>(conn_id: Uuid, messenger: M, player: Player) {
    let login_success = packet::LoginSuccess {
        uuid: player.uuid.to_hyphenated().to_string(),
        username: player.name,
    };
    messenger.send_packet(conn_id, Packet::LoginSuccess(login_success));
}

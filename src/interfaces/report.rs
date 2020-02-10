use super::block::BlockState;

use super::messenger::{Messenger, SubscriberType};
use super::patchwork::PatchworkState;
use super::player::PlayerState;

use uuid::Uuid;

pub fn report<M: Messenger, B: BlockState, PA: PatchworkState, P: PlayerState>(
    conn_id: Uuid,
    _messenger: M,
    player_state: P,
    block_state: B,
    patchwork_state: PA,
    subscriber_type: SubscriberType,
) {
    block_state.report(conn_id);
    player_state.report(conn_id);
    if let SubscriberType::Local = subscriber_type {
        patchwork_state.report();
    }
}

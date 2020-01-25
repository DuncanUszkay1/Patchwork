use super::interfaces::connection::Operations;
use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::PacketProcessor;
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::PlayerState;

use std::sync::mpsc::{Receiver, Sender};

pub fn start<
    M: Messenger + Clone,
    P: PlayerState + Clone,
    PA: PatchworkState + Clone,
    PP: 'static + PacketProcessor + Clone + Send,
>(
    receiver: Receiver<Operations>,
    _sender: Sender<Operations>,
    messenger: M,
    player_state: P,
    _patchwork_state: PA,
    _packet_processor: PP,
) {
    while let Ok(msg) = receiver.recv() {
        match msg {
            Operations::Close(msg) => {
                messenger.close(msg.conn_id);
                player_state.delete_player(msg.conn_id);
            }
        }
    }
}

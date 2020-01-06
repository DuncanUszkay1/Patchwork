use super::interfaces::block::BlockState;
use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::PacketProcessorOperations;
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::PlayerState;

use super::packet::{read, translate};
use super::packet_handlers::packet_router;
use super::translation::{TranslationInfo, TranslationUpdates};
use std::collections::HashMap;

use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub fn start_inbound<
    M: Messenger + Clone,
    P: PlayerState + Clone,
    PA: PatchworkState + Clone,
    B: BlockState + Clone,
>(
    receiver: Receiver<PacketProcessorOperations>,
    _sender: Sender<PacketProcessorOperations>,
    messenger: M,
    player_state: P,
    block_state: B,
    patchwork_state: PA,
) {
    let mut translation_data = HashMap::<Uuid, TranslationInfo>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PacketProcessorOperations::Inbound(msg) => {
                trace!("Received packet from conn_id {:?}", msg.conn_id);
                let translation_data = translation_data
                    .entry(msg.conn_id)
                    .or_insert_with(TranslationInfo::new);

                let packet = read(&mut msg.cursor.clone(), translation_data.state);
                let packet = translate(packet, translation_data.clone());
                let translation_update = packet_router::route_packet(
                    packet,
                    translation_data.state,
                    msg.conn_id,
                    messenger.clone(),
                    player_state.clone(),
                    block_state.clone(),
                    patchwork_state.clone(),
                );
                match translation_update {
                    TranslationUpdates::NoChange => {}
                    _ => {
                        trace!(
                            "Incoming translation update {:?} for conn_id {:?}",
                            translation_update,
                            msg.conn_id
                        );
                    }
                }
                translation_data.update(&translation_update);
            }
            PacketProcessorOperations::SetTranslationData(msg) => {
                trace!(
                    "Applying translation updates {:?} to {:?}",
                    msg.updates,
                    msg.conn_id
                );
                let data = translation_data
                    .entry(msg.conn_id)
                    .or_insert_with(TranslationInfo::new);

                msg.updates.iter().for_each(|update| {
                    data.update(update);
                })
            }
        }
    }
}

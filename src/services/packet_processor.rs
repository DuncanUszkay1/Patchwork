use super::interfaces::block::BlockState;
use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::Operations;
use super::interfaces::patchwork::PatchworkState;
use super::interfaces::player::PlayerState;

use super::packet::{read, translate, Packet};
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
    receiver: Receiver<Operations>,
    _sender: Sender<Operations>,
    messenger: M,
    player_state: P,
    block_state: B,
    patchwork_state: PA,
    test_sender: Option<std::sync::mpsc::Sender<(i32, Packet)>>,
) {
    let mut translation_data = HashMap::<Uuid, TranslationInfo>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            Operations::Inbound(msg) => {
                trace!("Received packet from conn_id {:?}", msg.conn_id);
                let translation_data = translation_data
                    .entry(msg.conn_id)
                    .or_insert_with(TranslationInfo::new);

                let packet = read(&mut msg.cursor.clone(), translation_data.state);
                let packet = translate(packet, translation_data.clone());

                // Send raw packet info if we provided a channel
                let test_sender_clone = test_sender.clone();
                if let Some(test_sender_clone) = test_sender_clone {
                    test_sender_clone
                        .send((translation_data.state, packet.clone()))
                        .expect("Failed to send packet to channel");
                }

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
            Operations::SetTranslationData(msg) => {
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

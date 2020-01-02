use super::game_state::block::BlockState;
use super::game_state::patchwork::PatchworkState;
use super::game_state::player::PlayerState;
use super::interfaces::messenger::Messenger;
use super::packet::{read, translate};
use super::packet_router;
use super::translation::{TranslationInfo, TranslationUpdates};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub trait PacketProcessor {
    fn inbound(&self, conn_id: Uuid, cursor: Cursor<Vec<u8>>);
    fn set_translation_data(&self, conn_id: Uuid, updates: Vec<TranslationUpdates>);
}

impl PacketProcessor for Sender<PacketProcessorOperations> {
    fn inbound(&self, conn_id: Uuid, cursor: Cursor<Vec<u8>>) {
        self.send(PacketProcessorOperations::Inbound(InboundPacketMessage {
            conn_id,
            cursor,
        }))
        .unwrap()
    }
    fn set_translation_data(&self, conn_id: Uuid, updates: Vec<TranslationUpdates>) {
        self.send(PacketProcessorOperations::SetTranslationData(
            TranslationDataMessage { conn_id, updates },
        ))
        .unwrap();
    }
}

pub enum PacketProcessorOperations {
    Inbound(InboundPacketMessage),
    SetTranslationData(TranslationDataMessage),
}

#[derive(Debug)]
pub struct InboundPacketMessage {
    pub conn_id: Uuid,
    pub cursor: Cursor<Vec<u8>>,
}

#[derive(Debug)]
pub struct TranslationDataMessage {
    pub conn_id: Uuid,
    pub updates: Vec<TranslationUpdates>,
}

pub fn start_inbound<
    M: Messenger + Clone,
    P: PlayerState + Clone,
    PA: PatchworkState + Clone,
    B: BlockState + Clone,
>(
    receiver: Receiver<PacketProcessorOperations>,
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

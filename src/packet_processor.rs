use super::game_state::block::BlockStateOperations;
use super::game_state::patchwork::PatchworkStateOperations;
use super::game_state::player::PlayerStateOperations;
use super::messenger::MessengerOperations;
use super::packet::{read, translate};
use super::packet_router;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub enum PacketProcessorOperations {
    Inbound(InboundPacketMessage),
    SetTranslationData(TranslationDataMessage),
}

#[derive(Debug)]
pub enum TranslationUpdates {
    State(i32),
    EntityIdBlock(i32),
    XOrigin(i32),
    NoChange,
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

#[derive(Debug, Clone, Copy)]
pub struct TranslationInfo {
    pub state: i32,
    pub entity_id_block: i32,
    pub map: Map,
}

#[derive(Debug, Clone, Copy)]
pub struct Map {
    pub x_origin: i32,
    pub y_origin: i32,
}

impl TranslationInfo {
    pub fn new() -> TranslationInfo {
        TranslationInfo {
            state: 0,
            map: Map {
                x_origin: 0,
                y_origin: 0,
            },
            entity_id_block: 0,
        }
    }

    pub fn update(&mut self, param: &TranslationUpdates) {
        match param {
            TranslationUpdates::State(state) => {
                self.state = *state;
            }
            TranslationUpdates::EntityIdBlock(block) => {
                self.entity_id_block = *block;
            }
            TranslationUpdates::XOrigin(x) => {
                self.map.x_origin = *x;
            }
            TranslationUpdates::NoChange => {}
        }
    }
}

pub fn start_inbound(
    receiver: Receiver<PacketProcessorOperations>,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
    block_state: Sender<BlockStateOperations>,
    patchwork_state: Sender<PatchworkStateOperations>,
) {
    let mut translation_data = HashMap::<Uuid, TranslationInfo>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PacketProcessorOperations::Inbound(msg) => {
                let translation_data = translation_data
                    .entry(msg.conn_id)
                    .or_insert_with(TranslationInfo::new);

                let packet = read(&mut msg.cursor.clone(), translation_data.state);
                let packet = translate(packet, *translation_data);
                translation_data.update(&packet_router::route_packet(
                    packet,
                    translation_data.state,
                    msg.conn_id,
                    messenger.clone(),
                    player_state.clone(),
                    block_state.clone(),
                    patchwork_state.clone(),
                ));
            }
            PacketProcessorOperations::SetTranslationData(msg) => {
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

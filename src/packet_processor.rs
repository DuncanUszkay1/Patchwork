use super::game_state::patchwork::PatchworkStateOperations;
use super::game_state::player::PlayerStateOperations;
use super::messenger::MessengerOperations;
use super::packet::read;
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
    pub update: TranslationUpdates,
}

#[derive(Debug)]
struct TranslationInfo {
    pub state: i32,
    pub map: Option<Map>,
}

#[derive(Debug)]
struct Map {
    pub x_origin: i32,
    pub y_origin: i32,
}

impl TranslationInfo {
    pub fn new() -> TranslationInfo {
        TranslationInfo {
            state: 0,
            map: None,
        }
    }

    pub fn update(&mut self, params: TranslationUpdates) {
        if let TranslationUpdates::State(state) = params {
            self.state = state;
        }
    }
}

pub fn start_inbound(
    receiver: Receiver<PacketProcessorOperations>,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
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
                translation_data.update(packet_router::route_packet(
                    packet,
                    translation_data.state,
                    msg.conn_id,
                    messenger.clone(),
                    player_state.clone(),
                    patchwork_state.clone(),
                ));
            }
            PacketProcessorOperations::SetTranslationData(msg) => {
                translation_data
                    .entry(msg.conn_id)
                    .or_insert_with(TranslationInfo::new)
                    .update(msg.update);
            }
        }
    }
}

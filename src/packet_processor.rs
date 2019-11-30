use super::game_state::patchwork::PatchworkStateOperations;
use super::game_state::player::PlayerStateOperations;
use super::messenger::MessengerOperations;
use super::packet::{Packet, read};
use super::packet_router;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

const ENTITY_ID_BLOCK_SIZE: i32 = 1000;

pub enum PacketProcessorOperations {
    Inbound(InboundPacketMessage),
    SetTranslationData(TranslationDataMessage),
}

#[derive(Debug)]
pub enum TranslationUpdates {
    State(i32),
    EntityIdBlock(i32),
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

#[derive(Debug, Clone, Copy)]
struct TranslationInfo {
    pub state: i32,
    pub entity_id_block: i32,
    pub map: Option<Map>,
}

#[derive(Debug, Clone, Copy)]
struct Map {
    pub x_origin: i32,
    pub y_origin: i32,
}

impl TranslationInfo {
    pub fn new() -> TranslationInfo {
        TranslationInfo {
            state: 0,
            map: None,
            entity_id_block: 0
        }
    }

    pub fn update(&mut self, params: TranslationUpdates) {
        match params {
            TranslationUpdates::State(state) => {
                self.state = state;
            }
            TranslationUpdates::EntityIdBlock(block) => {
                self.entity_id_block = block;
            }
            TranslationUpdates::NoChange => {}
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
                let packet = translate(packet, *translation_data);
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

fn translate(packet: Packet, translation_data: TranslationInfo) -> Packet {
    match packet {
        Packet::SpawnPlayer(spawn_player) => {
            let mut translated_packet = spawn_player;
            translated_packet.entity_id = translate_entity_id(translated_packet.entity_id, translation_data);
            Packet::SpawnPlayer(translated_packet)
        }
        _ => packet
    }
}

fn translate_entity_id(entity_id: i32, translation_data: TranslationInfo) -> i32 {
    entity_id + (translation_data.entity_id_block * ENTITY_ID_BLOCK_SIZE)
}
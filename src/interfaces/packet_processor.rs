use super::translation::TranslationUpdates;

use std::io::Cursor;
use std::sync::mpsc::Sender;
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

use super::translation::TranslationUpdates;

use std::io::Cursor;
use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(
    PacketProcessor,
    (Inbound, inbound, [conn_id: Uuid, cursor: Cursor<Vec<u8>>]),
    (
        SetTranslationData,
        set_translation_data,
        [conn_id: Uuid, updates: Vec<TranslationUpdates>]
    )
);

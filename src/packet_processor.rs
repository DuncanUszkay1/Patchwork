use super::game_state::player::{PlayerStateOperations};
use super::packet_router;
use super::messenger::{MessengerOperations};
use super::packet::read;
use std::collections::HashMap;
use std::io::{Cursor};
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub enum PacketProcessorOperations {
    Inbound(InboundPacketMessage)
}

#[derive(Debug)]
pub struct InboundPacketMessage {
    pub conn_id: Uuid,
    pub cursor: Cursor<Vec<u8>>
}

pub fn start_inbound(
    receiver: Receiver<PacketProcessorOperations>,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>
) {
    let mut state_map = HashMap::<Uuid, i32>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PacketProcessorOperations::Inbound(msg) => {
                let mut state = state_map.entry(msg.conn_id).or_insert(0);

                let packet = read(&mut msg.cursor.clone(), *state);
                packet_router::route_packet(
                    packet,
                    &mut state,
                    msg.conn_id,
                    messenger.clone(),
                    player_state.clone(),
                );
            }
        }
    }
}

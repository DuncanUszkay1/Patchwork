use super::messenger::{MessengerOperations, BroadcastPacketMessage};
use super::packet::{KeepAlive, Packet};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time;
use uuid::Uuid;

const KEEP_ALIVE_PERIOD: u64 = 15;
const KEEP_ALIVE_VALUE: i64 = 16;

pub enum KeepAliveOperations {
    New(NewKeepAliveConnectionMessage),
}

#[derive(Debug)]
pub struct NewKeepAliveConnectionMessage {
    pub conn_id: Uuid,
}

pub fn start_keep_alive(
    messenger: Sender<MessengerOperations>,
) {
    loop {
        sleep(time::Duration::from_secs(KEEP_ALIVE_PERIOD));
        broadcast_packet!(
            messenger,
            Packet::KeepAlive(KeepAlive {
                id: KEEP_ALIVE_VALUE
            }),
            None,
            false
        )
        .unwrap();
    }
}

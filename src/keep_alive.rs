use super::messenger::{BroadcastPacketMessage, MessengerOperations};
use super::packet::{KeepAlive, Packet};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time;

const KEEP_ALIVE_PERIOD: u64 = 15;
const KEEP_ALIVE_VALUE: i64 = 16;

pub fn start(_: Receiver<i32>, messenger: Sender<MessengerOperations>) {
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

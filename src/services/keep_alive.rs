use super::interfaces::messenger::{Messenger, SubscriberType};
use super::packet::{KeepAlive, Packet};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time;

const KEEP_ALIVE_PERIOD: u64 = 15;
const KEEP_ALIVE_VALUE: i64 = 16;

pub fn start<M: Messenger>(_: Receiver<i32>, _: Sender<i32>, messenger: M) {
    loop {
        sleep(time::Duration::from_secs(KEEP_ALIVE_PERIOD));
        messenger.broadcast(
            Packet::KeepAlive(KeepAlive {
                id: KEEP_ALIVE_VALUE,
            }),
            None,
            SubscriberType::Local,
        );
    }
}

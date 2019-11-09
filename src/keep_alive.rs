use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet::{KeepAlive, Packet};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time;

const KEEP_ALIVE_PERIOD: u64 = 15;
const KEEP_ALIVE_VALUE: i64 = 16;

pub enum KeepAliveOperations {
    New(NewKeepAliveConnectionMessage),
}

#[derive(Debug)]
pub struct NewKeepAliveConnectionMessage {
    pub conn_id: u64,
}

pub fn start_keep_alive(
    receiver: Receiver<KeepAliveOperations>,
    messenger: Sender<MessengerOperations>,
) {
    let mut conn_ids: Vec<u64> = Vec::new();

    loop {
        sleep(time::Duration::from_secs(KEEP_ALIVE_PERIOD));

        //after we wake up, add any new connections that were sent to us
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                KeepAliveOperations::New(msg) => {
                    conn_ids.push(msg.conn_id);
                }
            }
        }

        //send all the keep alives
        conn_ids.clone().into_iter().for_each(|conn_id| {
            send_packet!(
                messenger,
                conn_id,
                Packet::KeepAlive(KeepAlive {
                    id: KEEP_ALIVE_VALUE
                })
            )
            .unwrap();
        })
    }
}

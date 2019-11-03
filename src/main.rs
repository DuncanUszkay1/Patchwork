mod game_state;
mod messenger;
mod minecraft_protocol;
mod packet;
mod packet_router;
mod server;
use messenger::start_messenger;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let (msg_sender, msg_receiver) = channel();
    thread::spawn(move || start_messenger(msg_receiver));

    server::listen(msg_sender.clone());
}

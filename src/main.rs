#[macro_use]
mod messenger;
#[macro_use]
mod packet_macros;
mod game_state;
mod gameplay_router;
mod initiation_protocols;
mod keep_alive;
mod minecraft_protocol;
mod packet;
mod packet_router;
mod peer_conn_protocol;
mod server;
use game_state::player::start_player_state;
use keep_alive::start_keep_alive;
use messenger::start_messenger;
use std::env;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let (messenger_sender, messenger_receiver) = channel();
    let (keep_alive_sender, keep_alive_receiver) = channel();
    let (player_state_sender, player_state_receiver) = channel();

    thread::spawn(move || start_messenger(messenger_receiver, keep_alive_sender));

    let messenger_clone = messenger_sender.clone();
    thread::spawn(move || start_player_state(player_state_receiver, messenger_clone));

    let messenger_clone = messenger_sender.clone();
    thread::spawn(move || start_keep_alive(keep_alive_receiver, messenger_clone));

    let peer_ip_addr = String::from("127.0.0.1");
    let peer_port = env::var("PEER_PORT").unwrap().parse::<u16>().unwrap();
    peer_conn_protocol::send_p2p_handshake(peer_ip_addr, peer_port, messenger_sender.clone())
        .ok();

    server::listen(messenger_sender.clone(), player_state_sender.clone());
}
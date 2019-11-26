use super::minecraft_protocol::read_var_int;

use super::packet::read;
use super::packet_router;
use std::env;
use std::io;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use super::game_state::player::PlayerStateOperations;
use super::messenger::{MessengerOperations, NewConnectionMessage};
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub fn listen(messenger: Sender<MessengerOperations>, player_state: Sender<PlayerStateOperations>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", env::var("PORT").unwrap())).unwrap();

    for stream in listener.incoming() {
        println!("connection");
        let stream = stream.unwrap();
        let messenger_clone = messenger.clone();
        let player_state_clone = player_state.clone();
        thread::spawn(move || {
            handle_connection(stream, messenger_clone, player_state_clone);
        });
    }
}

pub fn handle_connection(
    mut stream: TcpStream,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
) {
    let mut state = 0;
    let conn_id = Uuid::new_v4();
    let stream_clone = stream.try_clone().unwrap();
    messenger
        .send(MessengerOperations::New(NewConnectionMessage {
            conn_id,
            socket: stream_clone,
        }))
        .unwrap();
    loop {
        match read_var_int(&mut stream) {
            Ok(length) => {
                let packet = read(&mut stream, state, length);
                packet_router::route_packet(
                    packet,
                    &mut state,
                    conn_id,
                    messenger.clone(),
                    player_state.clone(),
                );
            }
            Err(e) => {
                println!("conn closed due to {:?}", e);
                break;
            }
        }
    }
}

pub fn new_connection(peer_address: String, peer_port: u16) -> Result<TcpStream, io::Error> {
    let peer_info = format!("{}:{}", peer_address, peer_port.to_string());
    TcpStream::connect(peer_info)
}

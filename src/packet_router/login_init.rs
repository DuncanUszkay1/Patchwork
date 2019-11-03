use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet;
use super::packet::{read, write, Packet};
use super::game_state;
use super::game_state::spawn_player;
use std::sync::mpsc::Sender;

// Called upon user login
pub fn init_login(p: Packet, state: &mut u64, conn_id: i32, messenger: Sender<MessengerOperations>) {
    println!("Login protocol initiated :{:?}", p);
    match p.clone() {
        Packet::LoginStart(packet) => {
            println!("Joining ...");
            let login_packet = packet::LoginSuccess{
                uuid: "1c88a2d0-cdf1-4999-9dc1-3e2f696dde05".to_string(),
                username: packet.username,
            };
            println!("{:?}",login_packet);
            messenger.send(MessengerOperations::Send(SendPacketMessage {
                    conn_id,
                    packet: Packet::LoginSuccess(login_packet),
                })).unwrap();
            *state = 3; // Switches to play state
            println!("state={}",*state);
            game_state::spawn_player(conn_id, messenger);
        }
        _ => {println!("Login failed");}
    }
}

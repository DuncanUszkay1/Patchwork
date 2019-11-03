use super::messenger::{MessengerOperations, SendPacketMessage};
use super::packet;
use super::packet::Packet;
use std::sync::mpsc::Sender;

// Called when client pings the server
pub fn play(p: Packet, conn_id: i32, messenger: Sender<MessengerOperations>) {
    println!("Switched to play state");
}

pub fn spawn_player(conn_id: i32, messenger: Sender<MessengerOperations>) {
    // Joins the game
    println!("Joining the game ...");
    let p = packet::JoinGame{
            entity_id: 0,
            gamemode: 0,
            dimension: 0,
            difficulty: 0,
            max_players: 2,
            level_type: "DEFAULT".to_string(),
            reduced_debug_info: false,
        };
        println!("{:?}",p);
        messenger.send(MessengerOperations::Send(SendPacketMessage {
                conn_id,
                packet: Packet::JoinGame(p),
            })).unwrap();
    // Sets the player's position and camera
    println!("Seeting player's position and camera ...");
    let p = packet::PlayerPositionAndLook{
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            flags: 0,
            teleport_id: 0,
        };
        println!("{:?}",p);
        messenger.send(MessengerOperations::Send(SendPacketMessage {
                conn_id,
                packet: Packet::PlayerPositionAndLook(p),
            })).unwrap();
}
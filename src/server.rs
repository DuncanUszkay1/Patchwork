use super::minecraft_protocol::read_var_int;
use super::packet;
use super::packet::{read, write, Packet};
use std::net::TcpListener;
use std::net::TcpStream;

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        println!("connection");
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

const FAKE_RESPONSE: &str = "{\"version\": {\"name\": \"1.13.2\",\"protocol\": 404},\"players\": {\"max\": 100,\"online\": 5,\"sample\": [{\"name\": \"thinkofdeath\",\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\": {\"text\": \"Hello world\"},\"favicon\": \"data:image/png;base64,<data>\"}";

pub fn handle_connection(mut stream: TcpStream) {
    let mut state = 0;
    loop {
        //There's a lot of stuff jammed in this method right now- this will
        //need to be dispersed accross the system according to the design as our
        //next step
        match read_var_int(&mut stream) {
            Ok(_length) => {
                let packet = read(&mut stream, state);
                state = match packet.clone() {
                    Packet::Handshake(handshake) => handshake.next_state,
                    _ => state,
                };
                println!("Packet: {:?}", packet);
                match packet.clone() {
                    Packet::StatusRequest(_) => {
                        let status_response = packet::StatusResponse {
                            json_response: String::from(FAKE_RESPONSE),
                        };
                        write(&mut stream, Packet::StatusResponse(status_response));
                    }
                    Packet::Ping(ping) => {
                        let pong = packet::Pong {
                            payload: ping.payload,
                        };
                        write(&mut stream, Packet::Pong(pong));
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("conn closed due to {:?}", e);
                break;
            }
        }
    }
}

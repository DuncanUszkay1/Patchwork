use super::minecraft_protocol::read_var_int;

use super::packet::read;
use std::net::TcpListener;
use std::net::TcpStream;
use super::packet_router;

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut state = 0;
    for stream in listener.incoming() {
        println!("connection");
        let stream = stream.unwrap();

        handle_connection(stream, state);
    }
}


pub fn handle_connection(mut stream: TcpStream, mut state:u64) {
    
    loop {
        match read_var_int(&mut stream) {
            Ok(_length) => {
                let packet = read(&mut stream, state);
                packet_router::route_packet(packet, &mut state, &mut stream);        
            }
            Err(e) => {
                println!("conn closed due to {:?}", e);
                break;
            }
        }
    }
}
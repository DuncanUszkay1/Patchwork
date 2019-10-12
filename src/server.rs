use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;

mod packet_register;

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

pub fn handle_connection(mut stream: TcpStream) {
    let mut bytes: Vec<u8> = Vec::new();
    let size = stream.read_to_end(&mut bytes).unwrap();

    println!("Request: \n");

    let packet_id = packet_register::identify(&bytes);
    println!("Packet id : {}",packet_register::identify(&bytes));

    if(packet_id == 0){
        let hd_shake = packet_register::serialize_handshake(&bytes);
        println!("{:#?}", hd_shake);
    }
}
#[macro_use]
mod services;
mod constants;
mod interfaces;
mod models;
mod packet_handlers;
mod server;

use interfaces::patchwork::PatchworkState;

use services::instance::ServiceInstance;

use std::env;
use std::thread;

#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::{ConfigBuilder, LevelFilter, SimpleLogger};
extern crate serde;
extern crate serde_json;

const DEFAULT_LOGGING_LEVEL: LevelFilter = LevelFilter::Info;

fn main() {
    let level = match env::var("LOG") {
        Ok(level) => match level.as_str() {
            "info" => LevelFilter::Info,
            "trace" => LevelFilter::Trace,
            "error" => LevelFilter::Error,
            _ => DEFAULT_LOGGING_LEVEL,
        },
        Err(_) => DEFAULT_LOGGING_LEVEL,
    };

    let logger_config = ConfigBuilder::new()
        .set_max_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .build();

    SimpleLogger::init(level, logger_config).unwrap();

    define_services!(
        (
            module: services::player::start,
            name: player_state,
            dependencies: [messenger]
        ),
        (
            module: services::block::start,
            name: block_state,
            dependencies: [messenger]
        ),
        (
            module: services::patchwork::start,
            name: patchwork_state,
            dependencies: [messenger, inbound_packet_processor, player_state]
        ),
        (
            module: services::messenger::start,
            name: messenger,
            dependencies: []
        ),
        (
            module: services::packet_processor::start_inbound,
            name: inbound_packet_processor,
            dependencies: [messenger, player_state, block_state, patchwork_state]
        ),
        (
            module: services::connection::start,
            name: connection_service,
            dependencies: [messenger, player_state, patchwork_state, inbound_packet_processor]
        ),
        (
            module: services::keep_alive::start,
            name: keep_alive,
            dependencies: [messenger]
        )
    );

    trace!("Services Started");

    // the stuff below this should also probably be moved to a service model
    let peer_address = String::from("127.0.0.1");
    let peer_port = env::var("PEER_PORT").unwrap().parse::<u16>().unwrap();

    patchwork_state.sender().new_map(models::map::Peer {
        port: peer_port,
        address: peer_address,
    });

    server::listen(
        inbound_packet_processor.sender(),
        connection_service.sender(),
        messenger.sender(),
    );
}

#[cfg(test)]
mod tests {
    use crate::interfaces::connection::ConnectionService;
    use crate::interfaces::messenger::Messenger;
    use crate::interfaces::packet_processor::PacketProcessor;
    use crate::models::minecraft_protocol::MinecraftProtocolReader;
    use crate::*;

    fn start_trace() {
        let logger_config = ConfigBuilder::new()
            .set_max_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .build();
        SimpleLogger::init(LevelFilter::Trace, logger_config).unwrap();
    }

    fn handle_connection<M: Messenger, PP: PacketProcessor, F: Fn()>(
        mut stream: std::net::TcpStream,
        inbound_packet_processor: PP,
        messenger: M,
        id: uuid::Uuid,
        on_closure: F,
        sx: std::sync::mpsc::Sender<i32>,
    ) {
        let stream_clone = stream.try_clone().expect("Failed to clone stream");
        messenger.new_connection(id, stream_clone);
        loop {
            match stream.try_read_var_int() {
                Ok(length) => {
                    sx.send(length).expect("Failed to send data to channel");
                }
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::UnexpectedEof => on_closure(),
                        std::io::ErrorKind::ConnectionReset => on_closure(),
                        _ => {
                            panic!("Connection closed due to {:?}", e);
                        }
                    };
                    break;
                }
            }
        }
    }

    #[test]
    fn test() {
        start_trace();

        define_services!(
            (
                module: services::player::start,
                name: player_state,
                dependencies: [messenger]
            ),
            (
                module: services::block::start,
                name: block_state,
                dependencies: [messenger]
            ),
            (
                module: services::patchwork::start,
                name: patchwork_state,
                dependencies: [messenger, inbound_packet_processor, player_state]
            ),
            (
                module: services::messenger::start,
                name: messenger,
                dependencies: []
            ),
            (
                module: services::packet_processor::start_inbound,
                name: inbound_packet_processor,
                dependencies: [messenger, player_state, block_state, patchwork_state]
            ),
            (
                module: services::connection::start,
                name: connection_service,
                dependencies: [messenger, player_state, patchwork_state, inbound_packet_processor]
            ),
            (
                module: services::keep_alive::start,
                name: keep_alive,
                dependencies: [messenger]
            )
        );
        trace!("Services Started");

        // Temporary, maybe set them through CLI
        std::env::set_var("PORT", "8600");
        std::env::set_var("PEER_PORT", "8601");

        let address = String::from("127.0.0.1");
        let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();
        let peer_port = std::env::var("PEER_PORT").unwrap().parse::<u16>().unwrap();

        patchwork_state.sender().new_map(models::map::Peer {
            port: peer_port.clone(),
            address: address.clone(),
        });

        // Since servers handle connection in their own thread, create a channel
        // to retrieve information
        let (tx, rx) = std::sync::mpsc::channel();

        // Start a dummy peer server (although manually, it is the same code as server::listen(),
        // what's been added is the interaction with the channel)
        let connection_string = format!("{}:{}", address, peer_port);

        std::thread::spawn({
            let connection_string = connection_string.clone();
            let inbound_packet_processor = inbound_packet_processor.sender().clone();
            let messenger = messenger.sender().clone();
            let connection_service = connection_service.sender().clone();
            let tx = tx.clone();
            move || {
                let listener = std::net::TcpListener::bind(connection_string.clone())
                    .expect("Failed to bind socket");
                trace!("Listening on {:?}", connection_string);

                for stream in listener.incoming() {
                    let stream = stream.expect("Failed to connect to client");
                    let inbound_packet_processor_clone = inbound_packet_processor.clone();
                    let messenger_clone = messenger.clone();
                    let connection_service_clone = connection_service.clone();
                    let id = uuid::Uuid::new_v4();
                    let tx = tx.clone();
                    thread::spawn(move || {
                        handle_connection(
                            stream,
                            inbound_packet_processor_clone,
                            messenger_clone,
                            id,
                            || connection_service_clone.close(id),
                            tx,
                        );
                    });
                }
            }
        });

        // Start a proper server on its own thread
        std::thread::spawn({
            let inbound_packet_processor = inbound_packet_processor.sender().clone();
            let messenger = messenger.sender().clone();
            let connection_service = connection_service.sender().clone();
            move || {
                server::listen(inbound_packet_processor.clone(),
                               connection_service.clone(),
                               messenger.clone());
            }
        });

        // Ensure some delay to let the server launch
        std::thread::sleep(std::time::Duration::from_millis(100));
        match std::net::TcpStream::connect(connection_string.clone()) {
            Ok(stream) => {
                trace!("Connection successful to {}", connection_string);
                let length: i32 = rx.recv().expect("Failed to receive data from channel");
                trace!("Received packet length {}", length);
                assert!(length == 7);
                stream
                    .shutdown(std::net::Shutdown::Both)
                    .expect("Failed to shutdown connection"); // Everything went OK, close server and client
            }
            Err(_why) => {
                panic!("Failed to connect to {}", connection_string);
            }
        }
    }
}

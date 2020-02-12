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
            dependencies: [messenger, player_state, block_state, patchwork_state],
            extras: [None]
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
    use crate::*;

    fn start_trace() {
        let logger_config = ConfigBuilder::new()
            .set_max_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .build();
        SimpleLogger::init(LevelFilter::Trace, logger_config).unwrap();
    }

    #[test]
    fn test() {
        start_trace();

        // Since servers handle connection in their own thread, create a channel
        // to retrieve information
        let (router_sender, router_receiver) = std::sync::mpsc::channel();
        let optional_router_sender = Some(router_sender.clone());

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
                dependencies: [messenger, player_state, block_state, patchwork_state],
                extras: [optional_router_sender]
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
        let peer_port = std::env::var("PEER_PORT").unwrap().parse::<u16>().unwrap();

        patchwork_state.sender().new_map(models::map::Peer {
            port: peer_port,
            address: peer_address,
        });

        std::thread::spawn(move || {
            server::listen(
                inbound_packet_processor.sender(),
                connection_service.sender(),
                messenger.sender(),
            );
        });

        while let Ok((state, packet)) = router_receiver.recv() {
            trace!("==[Received]== {:?}, {:?}", state, packet);
        }
    }
}

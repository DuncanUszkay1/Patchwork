#[macro_use]
mod services;
mod models;
mod packet_handlers;
mod server;

use services::game_state::patchwork::{NewMapMessage, PatchworkStateOperations};

use services::instance::ServiceInstance;

use std::env;
use std::thread;

#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::{Config, LevelFilter, SimpleLogger};

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

    SimpleLogger::init(level, Config::default()).unwrap();

    define_services!(
        (
            module: services::game_state::player::start,
            name: player_state,
            dependencies: [messenger]
        ),
        (
            module: services::game_state::block::start,
            name: block_state,
            dependencies: [messenger]
        ),
        (
            module: services::game_state::patchwork::start,
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
            module: services::keep_alive::start,
            name: keep_alive,
            dependencies: [messenger]
        )
    );

    trace!("Services Started");

    // the stuff below this should also probably be moved to a service model
    let peer_address = String::from("127.0.0.1");
    let peer_port = env::var("PEER_PORT").unwrap().parse::<u16>().unwrap();

    patchwork_state
        .sender()
        .send(PatchworkStateOperations::New(NewMapMessage {
            peer: models::map::Peer {
                port: peer_port,
                address: peer_address,
            },
        }))
        .unwrap();

    server::listen(inbound_packet_processor.sender(), messenger.sender());
}

#[macro_use]
mod services;
mod models;
mod packet_handlers;
mod server;

use game_state::patchwork::{NewMapMessage, PatchworkStateOperations};

use packet_handlers::packet_router;

use models::map;
use models::minecraft_types;
use models::packet;
use models::translation;

use services::game_state;
use services::instance::ServiceInstance;
use services::keep_alive;
use services::messenger;
use services::packet_processor;

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
            module: game_state::player::start,
            name: player_state,
            dependencies: [messenger]
        ),
        (
            module: game_state::block::start,
            name: block_state,
            dependencies: [messenger]
        ),
        (
            module: game_state::patchwork::start,
            name: patchwork_state,
            dependencies: [messenger, inbound_packet_processor, player_state]
        ),
        (
            module: messenger::start,
            name: messenger,
            dependencies: []
        ),
        (
            module: packet_processor::start_inbound,
            name: inbound_packet_processor,
            dependencies: [messenger, player_state, block_state, patchwork_state]
        ),
        (
            module: keep_alive::start,
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
            peer: map::Peer {
                port: peer_port,
                address: peer_address,
            },
        }))
        .unwrap();

    server::listen(inbound_packet_processor.sender(), messenger.sender());
}

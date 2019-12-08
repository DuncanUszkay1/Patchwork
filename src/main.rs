#[macro_use]
mod messenger;
#[macro_use]
mod packet_macros;
#[macro_use]
mod service;
mod game_state;
mod gameplay_router;
mod initiation_protocols;
mod keep_alive;
mod minecraft_protocol;
mod packet;
mod packet_processor;
mod packet_router;
mod server;

use game_state::patchwork::{NewMapMessage, PatchworkStateOperations, Peer};
use service::ServiceInstance;

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
            dependencies: [messenger, inbound_packet_processor]
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
            peer: Peer {
                port: peer_port,
                address: peer_address,
            },
        }))
        .unwrap();

    server::listen(inbound_packet_processor.sender(), messenger.sender());
}

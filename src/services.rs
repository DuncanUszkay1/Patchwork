// Services are event loop methods that take in structured messages and return nothing. They may
// contain their own state, and they hold senders for any services they must talk to downstream.
// They run in their own threads, and are initialized by the define_services macro defined in the
// instance module

#[macro_use]
pub mod instance;
#[macro_use]
pub mod messenger;
pub mod block;
pub mod connection;
pub mod keep_alive;
pub mod packet_processor;
pub mod patchwork;
pub mod player;

use super::constants;

use super::models::map;
use super::models::minecraft_types;
use super::models::packet;
use super::models::translation;

use super::interfaces;

use super::packet_handlers;
use super::server;

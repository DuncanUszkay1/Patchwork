mod minecraft_protocol;
mod packet;
mod server;
mod packet_router;

fn main() {
    server::listen();
}
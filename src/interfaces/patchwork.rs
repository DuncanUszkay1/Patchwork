use super::map::Peer;
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub trait PatchworkState {
    fn new_map(&self, peer: Peer);
    fn route_player_packet(&self, packet: Packet, conn_id: Uuid);
    fn report(&self);
}

impl PatchworkState for Sender<PatchworkStateOperations> {
    fn new_map(&self, peer: Peer) {
        self.send(PatchworkStateOperations::New(NewMapMessage { peer }))
            .unwrap();
    }

    fn route_player_packet(&self, packet: Packet, conn_id: Uuid) {
        self.send(PatchworkStateOperations::RoutePlayerPacket(RouteMessage {
            packet,
            conn_id,
        }))
        .unwrap();
    }

    fn report(&self) {
        self.send(PatchworkStateOperations::Report).unwrap();
    }
}

pub enum PatchworkStateOperations {
    New(NewMapMessage),
    RoutePlayerPacket(RouteMessage),
    Report,
}

#[derive(Debug)]
pub struct NewMapMessage {
    pub peer: Peer,
}

#[derive(Debug, Clone)]
pub struct RouteMessage {
    pub packet: Packet,
    pub conn_id: Uuid,
}

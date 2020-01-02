use super::map::Map;
use super::packet::Packet;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub trait Messenger {
    fn send_packet(&self, conn_id: Uuid, packet: Packet);
    fn broadcast_packet(&self, packet: Packet, source_conn_id: Option<Uuid>, local: bool);
    fn subscribe(&self, conn_id: Uuid, typ: SubscriberType);
    fn new_connection(&self, conn_id: Uuid, socket: TcpStream);
    fn update_translation(&self, conn_id: Uuid, map: Map);
}

impl Messenger for Sender<MessengerOperations> {
    fn send_packet(&self, conn_id: Uuid, packet: Packet) {
        self.send(MessengerOperations::Send(SendPacketMessage {
            conn_id,
            packet,
        }))
        .unwrap();
    }

    fn broadcast_packet(&self, packet: Packet, source_conn_id: Option<Uuid>, local: bool) {
        self.send(MessengerOperations::Broadcast(BroadcastPacketMessage {
            packet,
            source_conn_id,
            local,
        }))
        .unwrap();
    }

    fn subscribe(&self, conn_id: Uuid, typ: SubscriberType) {
        self.send(MessengerOperations::Subscribe(SubscribeMessage {
            conn_id,
            typ,
        }))
        .unwrap();
    }

    fn new_connection(&self, conn_id: Uuid, socket: TcpStream) {
        self.send(MessengerOperations::New(NewConnectionMessage {
            conn_id,
            socket,
        }))
        .unwrap();
    }

    fn update_translation(&self, conn_id: Uuid, map: Map) {
        self.send(MessengerOperations::UpdateTranslation(
            UpdateTranslationMessage { conn_id, map },
        ))
        .unwrap();
    }
}

pub enum MessengerOperations {
    Send(SendPacketMessage),
    Broadcast(BroadcastPacketMessage),
    Subscribe(SubscribeMessage),
    New(NewConnectionMessage),
    UpdateTranslation(UpdateTranslationMessage),
}

#[derive(Debug)]
pub struct SendPacketMessage {
    pub conn_id: Uuid,
    pub packet: Packet,
}

#[derive(Debug)]
pub struct UpdateTranslationMessage {
    pub conn_id: Uuid,
    pub map: Map,
}

#[derive(Debug)]
pub struct SubscribeMessage {
    pub conn_id: Uuid,
    pub typ: SubscriberType,
}

#[derive(Debug)]
pub enum SubscriberType {
    All,
    LocalOnly,
}

#[derive(Debug)]
pub struct BroadcastPacketMessage {
    pub packet: Packet,
    pub source_conn_id: Option<Uuid>,
    pub local: bool,
}

#[derive(Debug)]
pub struct NewConnectionMessage {
    pub conn_id: Uuid,
    pub socket: TcpStream,
}

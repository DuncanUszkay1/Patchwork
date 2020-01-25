use super::map::Map;
use super::packet::Packet;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(
    Messenger,
    (Send, send_packet, [conn_id: Uuid, packet: Packet]),
    (
        Broadcast,
        broadcast,
        [
            packet: Packet,
            source_conn_id: Option<Uuid>,
            subscriber_type: SubscriberType
        ]
    ),
    (Subscribe, subscribe, [conn_id: Uuid, typ: SubscriberType]),
    (New, new_connection, [conn_id: Uuid, socket: TcpStream]),
    (
        UpdateTranslation,
        update_translation,
        [conn_id: Uuid, map: Map]
    ),
    (Close, close, [conn_id: Uuid])
);

#[derive(Debug)]
pub enum SubscriberType {
    All,
    Local,
    Remote,
}

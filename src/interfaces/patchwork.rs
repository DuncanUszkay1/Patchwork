use super::map::{Peer, PeerConnection};
use super::packet::Packet;
use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(
    PatchworkState,
    (Report, report, []),
    (New, new_map, [peer: Peer]),
    (
        RoutePlayerPacket,
        route_player_packet,
        [packet: Packet, conn_id: Uuid]
    ),
    (
        ConnectMap,
        connect_map,
        [map_index: usize, peer_connection: PeerConnection]
    )
);

use super::gameplay_router;
use super::interfaces::messenger::Messenger;
use super::map::{Map, Peer, Position};
use super::packet;
use super::packet::Packet;
use super::packet_processor::PacketProcessor;
use super::player::PlayerState;
use super::server;
use std::collections::HashMap;
use std::io;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use uuid::Uuid;

pub const ENTITY_ID_BLOCK_SIZE: i32 = 1000;
pub const CHUNK_SIZE: i32 = 16;

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

pub fn start<
    M: 'static + Messenger + Clone + Send,
    P: PlayerState + Clone,
    PP: 'static + PacketProcessor + Clone + Send,
>(
    receiver: Receiver<PatchworkStateOperations>,
    messenger: M,
    inbound_packet_processor: PP,
    player_state: P,
) {
    let mut patchwork = Patchwork::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            PatchworkStateOperations::New(msg) => {
                trace!("Adding Peer Map for peer {:?}", msg.peer);
                patchwork.add_peer_map(
                    msg.peer,
                    messenger.clone(),
                    inbound_packet_processor.clone(),
                )
            }
            PatchworkStateOperations::RoutePlayerPacket(msg) => {
                let patchwork_clone = patchwork.clone();
                let anchor = patchwork
                    .player_anchors
                    .entry(msg.conn_id)
                    .or_insert(Anchor {
                        map_index: 0,
                        conn_id: None,
                    });
                if let Some(position) = extract_map_position(msg.clone().packet) {
                    let new_map_index = patchwork_clone.position_map_index(position);
                    if new_map_index != anchor.map_index {
                        *anchor = match &patchwork.maps[new_map_index].peer_connection {
                            Some(peer_connection) => Anchor::connect(
                                peer_connection.peer.clone(),
                                msg.conn_id,
                                new_map_index,
                                patchwork.maps[new_map_index].position.x,
                                messenger.clone(),
                                player_state.clone(),
                            )
                            .unwrap(),
                            None => Anchor {
                                conn_id: None,
                                map_index: new_map_index,
                            },
                        }
                    }
                }
                match &patchwork.maps[anchor.map_index].peer_connection {
                    Some(_) => match msg.packet {
                        Packet::Unknown => {}
                        _ => {
                            trace!(
                                "Routing packet from conn_id {:?} through anchor",
                                msg.conn_id
                            );
                            messenger.send_packet(anchor.conn_id.unwrap(), msg.packet);
                        }
                    },
                    None => {
                        trace!("Routing packet from conn_id {:?} locally", msg.conn_id);
                        gameplay_router::route_packet(
                            msg.packet,
                            msg.conn_id,
                            player_state.clone(),
                        );
                    }
                }
            }
            PatchworkStateOperations::Report => {
                trace!("Reporting patchwork state");
                patchwork.clone().report(messenger.clone());
            }
        }
    }
}

fn extract_map_position(packet: Packet) -> Option<Position> {
    match packet {
        Packet::PlayerPosition(packet) => Some(Position {
            x: (packet.x / 16.0) as i32,
            z: (packet.z / 16.0) as i32,
        }),
        Packet::PlayerPositionAndLook(packet) => Some(Position {
            x: (packet.x / 16.0) as i32,
            z: (packet.z / 16.0) as i32,
        }),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct Anchor {
    map_index: usize,
    conn_id: Option<Uuid>,
}

impl Anchor {
    pub fn connect<M: Messenger, P: PlayerState>(
        peer: Peer,
        local_conn_id: Uuid,
        map_index: usize,
        x_origin: i32,
        messenger: M,
        player_state: P,
    ) -> Result<Anchor, io::Error> {
        let conn_id = Uuid::new_v4();
        let stream = server::new_connection(peer.address.clone(), peer.port)?;
        messenger.new_connection(conn_id, stream.try_clone().unwrap());
        messenger.update_translation(conn_id, Map::new(Position { x: x_origin, z: 0 }, 0));
        messenger.send_packet(
            conn_id,
            Packet::Handshake(packet::Handshake {
                protocol_version: 404,
                server_address: String::from(""), //Neither of these fields are actually used
                server_port: 0,
                next_state: 4,
            }),
        );
        player_state.cross_border(local_conn_id, conn_id);
        Ok(Anchor {
            map_index,
            conn_id: Some(conn_id),
        })
    }
}

#[derive(Debug, Clone)]
struct Patchwork {
    pub maps: Vec<Map>,
    pub player_anchors: HashMap<Uuid, Anchor>,
}

impl Patchwork {
    pub fn new() -> Patchwork {
        let mut patchwork = Patchwork {
            maps: Vec::new(),
            player_anchors: HashMap::new(),
        };
        patchwork.create_local_map();
        patchwork
    }

    pub fn create_local_map(&mut self) {
        self.maps
            .push(Map::new(self.next_position(), self.next_entity_id_block()));
    }

    pub fn position_map_index(self, position: Position) -> usize {
        self.maps
            .into_iter()
            .position(|map| map.position == position)
            .expect("Could not find map for position")
    }

    pub fn add_peer_map<
        M: 'static + Messenger + Send + Clone,
        PP: 'static + PacketProcessor + Send + Clone,
    >(
        &mut self,
        peer: Peer,
        messenger: M,
        inbound_packet_processor: PP,
    ) {
        if let Ok(map) = Map::new(self.next_position(), self.next_entity_id_block()).connect(
            messenger,
            inbound_packet_processor,
            peer,
        ) {
            self.maps.push(map);
        }
    }

    pub fn report<M: Messenger + Clone>(self, messenger: M) {
        self.maps
            .into_iter()
            .for_each(|map| map.report(messenger.clone()));
    }

    // get the next block of size 1000 entity ids assigned to this map
    fn next_entity_id_block(&self) -> i32 {
        self.maps.len() as i32
    }

    // For now, just line up all the maps in a row
    fn next_position(&self) -> Position {
        let len = self.maps.len() as i32;
        Position { x: len, z: 0 }
    }
}

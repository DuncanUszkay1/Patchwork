use super::interfaces::block::BlockState;
use super::interfaces::messenger::Messenger;
use super::interfaces::packet_processor::PacketProcessor;
use super::interfaces::patchwork::Operations;
use super::interfaces::player::{PlayerState, Position as PlayerPosition};
use super::map::{Map, Peer, PeerConnection, Position};
use super::packet;
use super::packet::Packet;
use super::packet_handlers::gameplay_router;
use super::server;

use std::collections::HashMap;
use std::io;
use std::sync::mpsc::{Receiver, Sender};

use uuid::Uuid;

pub fn start<
    M: 'static + Messenger + Clone + Send,
    P: PlayerState + Clone,
    PP: 'static + PacketProcessor + Clone + Send,
    B: BlockState + Clone,
>(
    receiver: Receiver<Operations>,
    sender: Sender<Operations>,
    messenger: M,
    inbound_packet_processor: PP,
    player_state: P,
    block_state: B,
) {
    let mut patchwork = Patchwork::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            Operations::New(msg) => {
                trace!("Adding Peer Map for peer {:?}", msg.peer);
                patchwork.add_peer_map(
                    msg.peer,
                    messenger.clone(),
                    inbound_packet_processor.clone(),
                    sender.clone(),
                )
            }
            Operations::ConnectMap(msg) => {
                patchwork.connect_map(msg.map_index, msg.peer_connection, messenger.clone());
            }
            Operations::RoutePlayerPacket(msg) => {
                let patchwork_clone = patchwork.clone();
                let anchor = patchwork
                    .player_anchors
                    .entry(msg.conn_id)
                    .or_insert(Anchor {
                        map_index: 0,
                        conn_id: None,
                    });
                if let Some(position) = extract_target_position((&msg.packet).clone()) {
                    let target_map_index = patchwork_clone.clone().position_map_index(position);
                    if target_map_index != anchor.map_index {
                        messenger.send_packet(
                            msg.conn_id,
                            Packet::ClientboundChatMessage(packet::ClientboundChatMessage {
                                message: String::from(
                                    "{\"text\":\"You must walk onto that map first before interacting with it\",\"bold\": \"true\"}",
                                ),
                                position: 1
                            }),
                        );
                        patchwork.clone().report(messenger.clone());
                        continue;
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
                            player_state.anchored_move_and_look(
                                msg.conn_id,
                                extract_player_position((&msg.packet).clone()),
                                None,
                            );
                            messenger.send_packet(anchor.conn_id.unwrap(), msg.packet.clone());
                        }
                    },
                    None => {
                        trace!("Routing packet from conn_id {:?} locally", msg.conn_id);
                        gameplay_router::route_packet(
                            msg.packet.clone(),
                            msg.conn_id,
                            player_state.clone(),
                            sender.clone(),
                            block_state.clone(),
                        );
                    }
                }
                if let Some(position) = extract_map_position((&msg.packet).clone()) {
                    let new_map_index = patchwork_clone.position_map_index(position);
                    if new_map_index != anchor.map_index {
                        anchor.disconnect(messenger.clone());
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
                            None => {
                                gameplay_router::route_packet(
                                    msg.packet.clone(),
                                    msg.conn_id,
                                    player_state.clone(),
                                    sender.clone(),
                                    block_state.clone(),
                                );
                                if patchwork.maps[anchor.map_index].peer_connection.is_some() {
                                    player_state.reintroduce(msg.conn_id);
                                }
                                Anchor {
                                    conn_id: None,
                                    map_index: new_map_index,
                                }
                            }
                        }
                    }
                }
            }
            Operations::Report(_) => {
                trace!("Reporting patchwork state");
                patchwork.clone().report(messenger.clone());
            }
        }
    }
}

fn extract_target_position(packet: Packet) -> Option<Position> {
    match packet {
        Packet::PlayerDigging(packet) => Some(Position {
            x: (packet.location.x as f32 / 16.0) as i32,
            z: (packet.location.z as f32 / 16.0) as i32,
        }),
        Packet::PlayerBlockPlacement(packet) => Some(Position {
            x: (packet.location.x as f32 / 16.0) as i32,
            z: (packet.location.z as f32 / 16.0) as i32,
        }),
        _ => None,
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

fn extract_player_position(packet: Packet) -> Option<PlayerPosition> {
    match packet {
        Packet::PlayerPosition(packet) => Some(PlayerPosition {
            x: packet.x,
            y: packet.feet_y,
            z: packet.z,
        }),
        Packet::PlayerPositionAndLook(packet) => Some(PlayerPosition {
            x: packet.x,
            y: packet.feet_y,
            z: packet.z,
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

    pub fn disconnect<M: Messenger>(&self, messenger: M) {
        if let Some(conn_id) = self.conn_id {
            messenger.close(conn_id);
        }
    }
}

#[derive(Debug, Clone)]
struct Patchwork {
    pub maps: Vec<Map>,
    pub player_anchors: HashMap<Uuid, Anchor>,
    pub next_position: Position,
    pub map_placement_direction: Direction,
}

impl Patchwork {
    pub fn new() -> Patchwork {
        let mut patchwork = Patchwork {
            maps: Vec::new(),
            player_anchors: HashMap::new(),
            next_position: Position { x: 0, z: 0 },
            map_placement_direction: Direction::ZNegative,
        };
        patchwork.create_local_map();
        patchwork
    }

    pub fn create_local_map(&mut self) {
        let next_position = self.next_position();
        self.maps
            .push(Map::new(next_position, self.next_entity_id_block()));
    }

    pub fn position_map_index(self, position: Position) -> usize {
        self.maps
            .into_iter()
            .position(|map| map.position == position)
            .expect("Could not find map for position")
    }

    pub fn connect_map<M: Messenger + Clone>(
        &mut self,
        map_index: usize,
        peer_connection: PeerConnection,
        messenger: M,
    ) {
        self.maps[map_index].peer_connection = Some(peer_connection);
        self.maps[map_index].report(messenger);
    }

    pub fn add_peer_map<
        M: 'static + Messenger + Send + Clone,
        PP: 'static + PacketProcessor + Send + Clone,
    >(
        &mut self,
        peer: Peer,
        messenger: M,
        inbound_packet_processor: PP,
        patchwork_state: Sender<Operations>,
    ) {
        let next_position = self.next_position();
        let map = Map::new(next_position, self.next_entity_id_block());
        self.maps.push(map.clone());
        map.connect(
            messenger,
            inbound_packet_processor,
            peer,
            patchwork_state,
            self.maps.len() - 1,
        );
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
    fn next_position(&mut self) -> Position {
        let ret = self.next_position;
        if self.next_position.x == self.next_position.z
            || (-self.next_position.x == self.next_position.z && self.next_position.x < 0)
            || (self.next_position.x > 0 && self.next_position.x == -self.next_position.z + 1)
        {
            self.map_placement_direction = self.map_placement_direction.turn();
        }
        self.next_position = self
            .map_placement_direction
            .shift_position(self.next_position);
        ret
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    XPositive,
    ZPositive,
    XNegative,
    ZNegative,
}

impl Direction {
    fn turn(self) -> Direction {
        match self {
            Direction::XPositive => Direction::ZPositive,
            Direction::ZPositive => Direction::XNegative,
            Direction::XNegative => Direction::ZNegative,
            Direction::ZNegative => Direction::XPositive,
        }
    }

    fn shift_position(self, position: Position) -> Position {
        match self {
            Direction::XPositive => Position {
                x: position.x + 1,
                z: position.z,
            },
            Direction::ZPositive => Position {
                x: position.x,
                z: position.z + 1,
            },
            Direction::XNegative => Position {
                x: position.x - 1,
                z: position.z,
            },
            Direction::ZNegative => Position {
                x: position.x,
                z: position.z - 1,
            },
        }
    }
}

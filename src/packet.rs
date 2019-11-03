#![allow(unused_variables)]
//Had to disable unused variables here since it wasn't working with packet::new
use super::minecraft_protocol::{write_var_int, MinecraftProtocolReader, MinecraftProtocolWriter};
use std::fmt::Debug;
use std::io::{Cursor, Read, Write};

// ideally this macro definition would be in a separate file to the actual data
// but I was running into issues with macro exports
// if we want to read in the packet definitions from json that'd probably fix
// that problem
macro_rules! packet_boilerplate {
    ( $( ( $state:expr, $name:ident, $id:expr, [
     $( ($fieldname:ident, $datatype:ident)),*
    ])),*) => (
        //Create an enum with a struct variant for each packet we've defined
        //and a special variant for a packet we haven't defined
        #[derive(Debug, Clone)]
        pub enum Packet {
            Unknown,
            $($name($name)),*
        }

        pub fn read<S: MinecraftProtocolReader + Read>(stream: &mut S, state: u64) -> Packet {
            //Length has already been discarded earlier, so we just need to read the
            //id in as a varint
            let id = stream.read_var_int();

            //call the initializer method of the packet class associated with
            //this state and packet id combination
            match (state,id) {
                $( ($state, $id) => { Packet::$name($name::new(stream)) } )*
                _ => {
                    println!("unknown packet state/id {:?}/{:?}", state, id);
                    Packet::Unknown
                }
            }
        }

        pub fn write<S: MinecraftProtocolWriter + Write>(stream: &mut S, packet: Packet) {
            //Write the ID and the values of the packet fields
            let mut cursor = Cursor::new(Vec::new());
            match packet {
                $(Packet::$name(packet) => {
                    write_var_int(&mut cursor, $name::ID);
                    packet.write(&mut cursor)
                })*
                _ => { panic!("I don't know how to write this packet") }
            }

            //Measure what we've written so far to determine packet length
            let size_vec = cursor.into_inner();
            let size = size_vec.len();

            //Write the length into a vector
            cursor = Cursor::new(Vec::new());
            write_var_int(&mut cursor, size as u64);

            //combine the length vector with the sizing vector to get
            //the full byte vector of the packet
            let mut byte_vec = cursor.into_inner();
            byte_vec.extend(size_vec);

            //Send the packet
            stream.write_all(&byte_vec).unwrap();
        }

        //Define the packet struct
        $(packet!{$state, $name, $id, [ $( ($fieldname, $datatype)),*]})*
    )
}

//Example: packet!(0, Handshake, 0, [ (somefield, VarInt) ] becomes:
//pub struct Handshake { somefield: u64 }
//impl Handshake {
//  pub fn new(stream) -> Handshake {
//    Handshake { somefiled: stream.read_var_int() }
//  }
//  pub fn write(self, stream) -> Handshake {
//    stream.write_var_int(self.somefield)
//  }
//}
macro_rules! packet {
    ($state:expr, $name:ident, $id:expr, [ $( ($fieldname:ident, $datatype:ident)),* ]) => (
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $fieldname: mc_to_rust_datatype!($datatype)),* }
        impl $name {
            const ID: u64 = $id;
            pub fn new<S: MinecraftProtocolReader>(stream: &mut S) -> $name {
                $name { $( $fieldname: read_packet_field!(stream, $datatype) ),* }
            }
            pub fn write<S: MinecraftProtocolWriter>(&self, stream: &mut S) {
                $( write_packet_field!(stream, self.$fieldname, $datatype) );*
            }
        }
    )
}

macro_rules! mc_to_rust_datatype {
    (VarInt) => {
        u64
    };
    (UShort) => {
        u16
    };
    (Long) => {
        i64
    };
    (String) => {
        String
    };
    (Int) => {
        i32
    };
    (Float) => {
        f32
    };
    (Double) => {
        f64
    };
    (Byte) => {
        i8
    };
    (UByte) => {
        u8
    };
    (Boolean) => {
        bool
    };
}

macro_rules! read_packet_field {
    ($stream:ident, VarInt) => {
        $stream.read_var_int()
    };
    ($stream:ident, UShort) => {
        $stream.read_unsigned_short()
    };
    ($stream:ident, Long) => {
        $stream.read_long()
    };
    ($stream:ident, String) => {
        $stream.read_string()
    };
    ($stream:ident, Int) => {
        $stream.read_int()
    };
    ($stream:ident, Float) => {
        $stream.read_float()
    };
    ($stream:ident, Double) => {
        $stream.read_double()
    };
    ($stream:ident, Byte) => {
        $stream.read_byte()
    };
    ($stream:ident, UByte) => {
        $stream.read_u_byte()
    };
    ($stream:ident, Boolean) => {
        $stream.read_boolean()
    };
}

macro_rules! write_packet_field {
    ($stream:ident, $value:expr, VarInt) => {
        $stream.write_var_int($value)
    };
    ($stream:ident, $value:expr, UShort) => {
        $stream.write_unsigned_short($value)
    };
    ($stream:ident, $value:expr, Long) => {
        $stream.write_long($value)
    };
    ($stream:ident, $value:expr, String) => {
        $stream.write_string($value.clone())
    };
    ($stream:ident, $value:expr, Int) => {
        $stream.write_int($value)
    };
    ($stream:ident, $value:expr, Float) => {
        $stream.write_float($value)
    };
    ($stream:ident, $value:expr, Double) => {
        $stream.write_double($value)
    };
    ($stream:ident, $value:expr, Byte) => {
        $stream.write_byte($value)
    };
    ($stream:ident, $value:expr, UByte) => {
        $stream.write_u_byte($value)
    };
    ($stream:ident, $value:expr, Boolean) => {
        $stream.write_boolean($value)
    };
}

// Format: (state, name, id, [ list of (field name, field type) ]
packet_boilerplate!(
    (
        0,
        Handshake,
        0,
        [
            (protocol_version, VarInt),
            (server_address, String),
            (server_port, UShort),
            (next_state, VarInt)
        ]
    ),
    (1, StatusRequest, 0, []),
    (1, Ping, 1, [(payload, Long)]),
    (11, Pong, 1, [(payload, Long)]),
    (11, StatusResponse, 0, [(json_response, String)]),
    (2, LoginStart, 0, [(username, String)]),
    (11, LoginSuccess, 2, [(uuid, String), (username, String)]),
    (
        11,
        JoinGame,
        0x25,
        [
            (entity_id, Int),
            (gamemode, UByte),
            (dimension, Int),
            (difficulty, UByte),
            (max_players, UByte),
            (level_type, String),
            (reduced_debug_info, Boolean)
        ]
    ),
    (
        3,
        PlayerPositionAndLook,
        0x32,
        [
            (x, Double),
            (y, Double),
            (z, Double),
            (yaw, Float),
            (pitch, Float),
            (flags, Byte),
            (teleport_id, VarInt)
        ]
    )
);

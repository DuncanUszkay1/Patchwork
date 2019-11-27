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

        pub fn read<S: MinecraftProtocolReader + Read>(stream: &mut S, state: i32, length: i32) -> Packet {
            //Read the entire packet into a vector first
            let vec: Vec<u8> = stream
                .bytes()
                .take(length as usize)
                .map(|r: Result<u8, _>| r.expect("packet was smaller than length field indicated!"))
                .collect();
            let mut cursor = Cursor::new(vec);

            //Length has already been discarded earlier, so we just need to read the
            //id in as a varint
            let id = cursor.read_var_int();

            //call the initializer method of the packet class associated with
            //this state and packet id combination
            match (state,id) {
                $( ($state, $id) => { Packet::$name($name::new(&mut cursor)) } )*
                _ => {
                    println!("unknown packet state/id {:?}/{:x}", state, id);
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
            write_var_int(&mut cursor, size as i32);

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
            const ID: i32 = $id;
            pub fn new<S: MinecraftProtocolReader>(stream: &mut S) -> $name {
                $name { $( $fieldname: read_packet_field!(stream, $datatype) ),* }
            }
            pub fn write<S: MinecraftProtocolWriter>(&self, stream: &mut S) {
                $( write_packet_field!(stream, self.$fieldname.clone(), $datatype) );*
            }
        }
    )
}

macro_rules! mc_to_rust_datatype {
    (VarInt) => {
        i32
    };
    (UShort) => {
        u16
    };
    (Short) => {
        i16
    };
    (Long) => {
        i64
    };
    (String) => {
        String
    };
    (u128) => {
        u128
    };
    (Long) => {
        i64
    };
    (Int) => {
        i32
    };
    (IntArray) => {
        Vec::<i32>
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
    (ChunkSection) => {
        ChunkSection
    };
}

macro_rules! read_packet_field {
    ($stream:ident, VarInt) => {
        $stream.read_var_int()
    };
    ($stream:ident, UShort) => {
        $stream.read_unsigned_short()
    };
    ($stream:ident, Short) => {
        $stream.read_short()
    };
    ($stream:ident, Long) => {
        $stream.read_long()
    };
    ($stream:ident, String) => {
        $stream.read_string()
    };
    ($stream:ident, u128) => {
        $stream.read_u_128()
    };
    ($stream:ident, Int) => {
        $stream.read_int()
    };
    ($stream:ident, IntArray) => {
        $stream.read_int_array()
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
    ($stream:ident, ChunkSection) => {
        $stream.read_chunk_section()
    };
}

macro_rules! write_packet_field {
    ($stream:ident, $value:expr, VarInt) => {
        $stream.write_var_int($value)
    };
    ($stream:ident, $value:expr, UShort) => {
        $stream.write_unsigned_short($value)
    };
    ($stream:ident, $value:expr, Short) => {
        $stream.write_short($value)
    };
    ($stream:ident, $value:expr, Long) => {
        $stream.write_long($value)
    };
    ($stream:ident, $value:expr, String) => {
        $stream.write_string($value.clone())
    };
    ($stream:ident, $value:expr, u128) => {
        $stream.write_u_128($value)
    };
    ($stream:ident, $value:expr, Int) => {
        $stream.write_int($value)
    };
    ($stream:ident, $value:expr, IntArray) => {
        $stream.write_int_array($value)
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
    ($stream:ident, $value:expr, ChunkSection) => {
        $stream.write_chunk_section($value)
    };
}

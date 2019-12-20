macro_rules! packet_boilerplate {
    ( $( ( $state:pat, $name:ident, $id:expr,
           [$(($fieldname:ident, $datatype:ident$(($($typearg:tt),*))* $(, $transtype:tt),* ) ),*]
    )),*) => (
        //Create an enum with a struct variant for each packet we've defined
        //and a special variant for a packet we haven't defined
        #[derive(Debug, Clone)]
        pub enum Packet {
            Unknown,
            $($name($name)),*
        }

        pub fn read<S: MinecraftProtocolReader + Read>(stream: &mut S, state: i32) -> Packet {
            let id = stream.read_var_int();

            //call the initializer method of the packet class associated with
            //this state and packet id combination
            match (state,id) {
                $( ($state, $id) => {
                    let packet = Packet::$name($name::new(stream));
                    if stream.bytes().next().is_some() {
                        panic!(
                            "Failed to read entire buffer for packet with id {:?} in state {:?}",
                            id,
                            state
                        );
                    }
                    packet
                } )*
                _ => {
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
                    packet.write_fields(&mut cursor)
                })*
                _ => { panic!("I don't know how to write this packet {:?}", packet) }
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

        pub fn translate(packet: Packet, translation_info: TranslationInfo) -> Packet {
            match packet {
                $(Packet::$name(packet) => {
                    Packet::$name(packet.translate(translation_info))
                })*
                Packet::Unknown => { Packet::Unknown }
            }
        }

        //Define the packet struct
        $(packet!{$name, $id, [ $( ( $fieldname, $datatype$(($($typearg),*))* $(, $transtype),* ) ),* ]})*
    )
}

macro_rules! packet {
    ($name:ident, $id:expr, [ $( ($fieldname:ident, $datatype:ident$(($($typearg:tt),*))* $(, $transtype:tt),* )),+]) => (
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $fieldname: mc_to_rust_datatype!($datatype$(($($typearg),*))*)),* }
        impl $name {
            const ID: i32 = $id;
            pub fn new<S: MinecraftProtocolReader>(stream: &mut S) -> $name {
                $name { $( $fieldname: read_packet_field!(stream, $datatype$(($($typearg),*))*) ),* }
            }
            pub fn write_fields<S: MinecraftProtocolWriter>(&self, stream: &mut S) {
                $( write_packet_field!(stream, self.$fieldname.clone(), $datatype$(($($typearg),*))*) );*
            }
            pub fn translate(&self, translation_data: TranslationInfo) -> $name {
                let mut translated = self.clone();
                $(translated.$fieldname = translate_packet_field!(
                        self.$fieldname.clone(),
                        translation_data
                        $(, $transtype ),*
                ); )*
                translated
            }
        }
    );
    ($name:ident, $id:expr, []) => (
        #[derive(Debug, Clone)]
        pub struct $name {}
        impl $name {
            const ID: i32 = $id;
            pub fn new<S: MinecraftProtocolReader>(stream: &mut S) -> $name {
                $name {}
            }
            pub fn write_fields<S: MinecraftProtocolWriter>(&self, stream: &mut S) {}
            pub fn translate(&self, translation_data: TranslationInfo) -> $name {
                self.clone()
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
    (Array($type:ident, $length:expr)) => {
        Vec::<mc_to_rust_datatype!($type)>
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
    ($stream:ident, Array($type:ident, $length:expr)) => {
        $stream.read_int_array($length)
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
    ($stream:ident, $value:expr, Array($type:ident, $length:expr)) => {
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

macro_rules! translate_packet_field {
    ($value:expr, $transdata:expr) => {
        $value
    };
    ($value:expr, $transdata:expr, EntityId) => {
        $value + ($transdata.entity_id_block * ENTITY_ID_BLOCK_SIZE)
    };
    ($value:expr, $transdata:expr, XChunk) => {
        $transdata.map.x_origin
    };
    ($value:expr, $transdata:expr, XEntity) => {
        $value + ($transdata.map.x_origin * CHUNK_SIZE) as f64
    };
}

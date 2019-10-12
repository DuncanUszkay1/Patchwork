use std::collections::HashMap;
use std::fmt::Debug;


pub fn identify(data: &Vec<u8>) -> u64{
    let mut i = 0;
    let length = read_var_int(&data,&mut i); // reads the first VarInt (length)
    let id = read_var_int(&data,&mut i);    // returns the second VarInt (id)

    id
}

pub fn serialize_handshake(data: &Vec<u8>) -> Handshake{
    let mut i = 0;
    let length = read_var_int(&data,&mut i);
    let id = read_var_int(&data,&mut i);
    let prot_version = read_var_int(&data,&mut i);
    // read string, unsigned short and varint enum not impl yet
    let srv_addr = String::from("127.0.0.1");
    let srv_port = 7878;
    let next_state = 2;

    Handshake::new(id,prot_version,srv_addr,srv_port,next_state)
}

pub fn read_var_int(data: &Vec<u8>, i: &mut usize) -> u64{
    let mut numRead = 0;
    let mut result:u8 = 0;

    let mut value = data[*i] & 0b01111111;
    
    result |= (value << (7 * numRead));
    numRead+=1;
    if numRead > 5 {
        println!("VarInt is too big");
    }
    *i += 1;

    while (data[*i] & 0b10000000) != 0 {
        value = data[*i] & 0b01111111;
        result |= (value << (7 * numRead));

        numRead+=1;
        if numRead > 5 {
            println!("VarInt is too big");
        }
        *i +=1;
    } 

    result.into()
}

#[derive(Debug)]
pub struct Handshake {
    id: u64, //VarInt
    protocol_version: u64, // VarInt
    server_address: String, //String (255)
    server_port: u16,  // Unsigned Short 
    next_state: u8,  // VarInt Enum
    packet_type: PacketType,
}

impl Handshake {
    fn new(id: u64,protocol_version:u64,server_address:String,server_port:u16,next_state:u8) -> Handshake {
        let mut fields_name: Vec<String> = Vec::new();
        let mut fields_type: Vec<DataTypes> = Vec::new();
        
        fields_name.push(String::from("id"));
        fields_name.push(String::from("protocol_version"));
        fields_name.push(String::from("server_address"));
        fields_name.push(String::from("server_port"));
        fields_name.push(String::from("next_state"));

        fields_type.push(DataTypes::VarInt);
        fields_type.push(DataTypes::VarInt);
        fields_type.push(DataTypes::StringN);
        fields_type.push(DataTypes::UShort);
        fields_type.push(DataTypes::XEnum);

        let pckt_type = PacketType::new(fields_name, fields_type);
        
        Handshake {
            id,
            protocol_version,
            server_address,
            server_port,
            next_state,
            packet_type:pckt_type,
        }
    }
}

#[derive(Debug)]
pub struct PacketType {
    fields: HashMap<String,DataTypes>,
}

impl PacketType {
    fn new(fields_name: Vec<String>, fields_type: Vec<DataTypes>) -> PacketType {
        let mut fields_map: HashMap<String,DataTypes> = HashMap::new();
        
        let mut i = 0;
        for f_name in &fields_name {
            fields_map.insert(String::from(f_name), fields_type[i]);
            i = i+1;
        }

        PacketType {
            fields: fields_map,
        }
    }
}

#[derive(Debug)]
#[derive(Copy,Clone)]
enum DataTypes {
    Boolean,   // True is encoded as 0x01, false as 0x00.
    Byte,      // An integer between -128 and 127 	Signed 8-bit integer, two's complement
    UByte,    // An integer between 0 and 255 	Unsigned 8-bit integer
    Short,     // An integer between -32768 and 32767 	Signed 16-bit integer, two's complement
    UShort,   // Unsigned 16-bit integer
    Int,       // An integer between -2147483648 and 2147483647 	Signed 32-bit integer, two's complement
    Long,      // Signed 64-bit integer, two's complement
    Float,     // single-precision 32-bit IEEE 754 floating point number 	
    Double, 	// double-precision 64-bit IEEE 754 floating point number 	
    StringN,	// A sequence of Unicode scalar values 	UTF-8 string prefixed with its size in bytes as a VarInt. Maximum length of n characters, which varies by context; up to n × 4 bytes can be used to encode n characters and both of those limits are checked.
    Chat,      // String with max length of 32767.
    Identifier,// String with max length of 32767.
    VarInt,    // u64; a two's complement signed 32-bit integer (MAX 5 bytes)
    VarLong,   // An integer between -9223372036854775808 and 9223372036854775807 	Variable-length data encoding a two's complement signed 64-bit integer; more info in their section
    EntityMetadata,   // Varies 	Miscellaneous information about an entity 	See Entities#Entity Metadata Format
    Slot, 	            // Varies 	An item stack in an inventory or container 	See Slot Data
    NbtTag, 	// Varies 	Depends on context 	See NBT
    Position, 	// An integer/block position: x (-33554432 to 33554431), y (-2048 to 2047), z (-33554432 to 33554431) 	x as a 26-bit integer, followed by y as a 12-bit integer, followed by z as a 26-bit integer (all signed, two's complement)
    Angle, 	 	// A rotation angle in steps of 1/256 of a full turn 	Whether or not this is signed does not matter, since the resulting angles are the same.
    Uuid, 	    // Encoded as an unsigned 128-bit integer (or two unsigned 64-bit integers: the most significant 64 bits and then the least significant 64 bits)
    OptionalX,// 0 or size of X 	A field of type X, or nothing 	Whether or not the field is present must be known from the context.
    XArray, 	// Zero or more fields of type X 	The count must be known from the context.
    XEnum, 	// A specific value from a given list 	The list of possible values and how each is encoded as an X must be known from the context. An invalid value sent by either side will usually result in the client being disconnected with an error or even crashing.
    ByteArray, // [] sequence of zero or more bytes, see context
}

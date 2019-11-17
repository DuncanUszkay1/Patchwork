extern crate byteorder;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Error, Read, Write};

pub trait MinecraftProtocolReader {
    fn read_unsigned_short(&mut self) -> u16;
    fn read_short(&mut self) -> i16;
    fn read_var_int(&mut self) -> u64;
    fn read_long(&mut self) -> i64;
    fn read_string(&mut self) -> String;
    fn read_u_128(&mut self) -> u128;
    fn read_int(&mut self) -> i32;
    fn read_int_array(&mut self) -> Vec<i32>;
    fn read_chunk_section(&mut self) -> ChunkSection;
    fn read_float(&mut self) -> f32;
    fn read_double(&mut self) -> f64;
    fn read_byte(&mut self) -> i8;
    fn read_u_byte(&mut self) -> u8;
    fn read_boolean(&mut self) -> bool;
}

pub trait MinecraftProtocolWriter {
    fn write_long(&mut self, v: i64);
    fn write_unsigned_short(&mut self, v: u16);
    fn write_short(&mut self, v: i16);
    fn write_var_int(&mut self, v: u64);
    fn write_string(&mut self, v: String);
    fn write_u_128(&mut self, v: u128);
    fn write_int(&mut self, v: i32);
    fn write_int_array(&mut self, v: Vec<i32>);
    fn write_chunk_section(&mut self, v: ChunkSection);
    fn write_float(&mut self, v: f32);
    fn write_double(&mut self, v: f64);
    fn write_byte(&mut self, v: i8);
    fn write_u_byte(&mut self, v: u8);
    fn write_boolean(&mut self, v: bool);
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub bits_per_block: u8, //always 14 until we implement palettes
    pub data_array_length: u64,
    pub block_ids: Vec<i32>,   //4096 block ids
    pub block_light: Vec<u64>, //2048 bytes (all 1s)
    pub sky_light: Vec<u64>,   //2048 bytes (all 1s)
}

pub fn read_var_int<S: Read>(stream: &mut S) -> Result<u64, Error> {
    let mut num_read = 0;
    let mut result: u64 = 0;

    loop {
        let value = u64::from(stream.read_u8()?);
        result |= (value & 0b0111_1111) << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            panic!("VarInt is too big");
        }
        if (value & 0b1000_0000) == 0 {
            break;
        }
    }

    Ok(result)
}

pub fn write_chunk_section<S: Write>(stream: &mut S, v: ChunkSection) {
    stream.write_u_byte(v.bits_per_block);
    stream.write_var_int(v.data_array_length);
    for _ in 0..896 {
        stream.write_long(1_000_000_000_000); //write all air blocks
    }
    for _ in 0..2048 {
        stream
            .write_u8(!0b0)
            .expect("could not write max block light"); //write max block light
    }
    for _ in 0..2048 {
        stream
            .write_u8(!0b0)
            .expect("could not write max sky light"); //write max sky light
    }
}

pub fn write_var_int<S: Write>(stream: &mut S, v: u64) {
    let mut value = v;
    loop {
        let mut temp = value & 0b0111_1111;
        value >>= 7;
        if value != 0 {
            temp |= 0b1000_0000;
        }
        stream.write_u8(temp as u8).unwrap();
        if value == 0 {
            break;
        }
    }
}

impl<T: Read> MinecraftProtocolReader for T {
    fn read_long(&mut self) -> i64 {
        self.read_i64::<BigEndian>().unwrap()
    }

    fn read_var_int(&mut self) -> u64 {
        read_var_int(self).unwrap()
    }

    fn read_unsigned_short(&mut self) -> u16 {
        self.read_u16::<BigEndian>().unwrap()
    }

    fn read_short(&mut self) -> i16 {
        self.read_i16::<BigEndian>().unwrap()
    }

    fn read_string(&mut self) -> String {
        let size = self.read_var_int();

        let mut buffer = vec![0; size as usize];
        self.read_exact(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    fn read_u_128(&mut self) -> u128 {
        self.read_u128::<BigEndian>().unwrap()
    }

    fn read_int(&mut self) -> i32 {
        self.read_i32::<BigEndian>().unwrap()
    }

    fn read_int_array(&mut self) -> Vec<i32> {
        unimplemented!();
    }

    fn read_float(&mut self) -> f32 {
        self.read_f32::<BigEndian>().unwrap()
    }

    fn read_chunk_section(&mut self) -> ChunkSection {
        unimplemented!();
    }

    fn read_double(&mut self) -> f64 {
        self.read_f64::<BigEndian>().unwrap()
    }

    fn read_byte(&mut self) -> i8 {
        self.read_i8().unwrap()
    }

    fn read_u_byte(&mut self) -> u8 {
        self.read_u8().unwrap()
    }

    fn read_boolean(&mut self) -> bool {
        match self.read_u8().unwrap() {
            1 => true,
            0 => false,
            _ => {
                println!("Error while unwrapping boolean");
                false
            }
        }
    }
}

impl<T: Write> MinecraftProtocolWriter for T {
    fn write_long(&mut self, v: i64) {
        self.write_i64::<BigEndian>(v).unwrap();
    }

    fn write_var_int(&mut self, v: u64) {
        write_var_int(self, v)
    }

    fn write_unsigned_short(&mut self, v: u16) {
        self.write_u16::<BigEndian>(v).unwrap()
    }

    fn write_short(&mut self, v: i16) {
        self.write_i16::<BigEndian>(v).unwrap()
    }

    fn write_string(&mut self, v: String) {
        let string_bytes = v.into_bytes();
        self.write_var_int(string_bytes.clone().into_iter().count() as u64);
        self.write_all(&string_bytes).unwrap();
    }

    fn write_u_128(&mut self, v: u128) {
        self.write_u128::<BigEndian>(v).unwrap();
    }

    fn write_int(&mut self, v: i32) {
        self.write_i32::<BigEndian>(v).unwrap();
    }

    fn write_int_array(&mut self, v: Vec<i32>) {
        v.iter()
            .for_each(|element| self.write_i32::<BigEndian>(*element).unwrap());
    }

    fn write_chunk_section(&mut self, v: ChunkSection) {
        write_chunk_section(self, v);
    }

    fn write_float(&mut self, v: f32) {
        self.write_f32::<BigEndian>(v).unwrap();
    }

    fn write_double(&mut self, v: f64) {
        self.write_f64::<BigEndian>(v).unwrap();
    }

    fn write_byte(&mut self, v: i8) {
        self.write_i8(v).unwrap();
    }

    fn write_u_byte(&mut self, v: u8) {
        self.write_u8(v).unwrap();
    }

    fn write_boolean(&mut self, v: bool) {
        if v {
            self.write_u8(1).unwrap()
        } else {
            self.write_u8(0).unwrap()
        }
    }
}

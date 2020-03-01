extern crate byteorder;

use super::minecraft_types::{BlockPosition, ChunkSection};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp::{max, min};
use std::io::{Error, Read, Write};

const PALETTE_SIZE: i64 = 14; // We don't define our own palette, so we just use the default all blocks palette which is 14 bits

pub trait MinecraftProtocolReader {
    fn read_unsigned_short(&mut self) -> u16;
    fn read_unsigned_long(&mut self) -> u64;
    fn read_short(&mut self) -> i16;
    fn read_var_int(&mut self) -> i32;
    fn try_read_var_int(&mut self) -> Result<i32, Error>;
    fn read_long(&mut self) -> i64;
    fn read_string(&mut self) -> String;
    fn read_u_128(&mut self) -> u128;
    fn read_int(&mut self) -> i32;
    fn read_int_array(&mut self, length: u32) -> Vec<i32>;
    fn read_var_int_array(&mut self, length: u32) -> Vec<i32>;
    fn read_chunk_section(&mut self) -> ChunkSection;
    fn read_float(&mut self) -> f32;
    fn read_double(&mut self) -> f64;
    fn read_byte(&mut self) -> i8;
    fn read_u_byte(&mut self) -> u8;
    fn read_boolean(&mut self) -> bool;
    fn read_position(&mut self) -> BlockPosition;
}

pub trait MinecraftProtocolWriter {
    fn write_long(&mut self, v: i64);
    fn write_unsigned_short(&mut self, v: u16);
    fn write_unsigned_long(&mut self, v: u64);
    fn write_short(&mut self, v: i16);
    fn write_var_int(&mut self, v: i32);
    fn write_string(&mut self, v: String);
    fn write_u_128(&mut self, v: u128);
    fn write_int(&mut self, v: i32);
    fn write_int_array(&mut self, v: Vec<i32>);
    fn write_var_int_array(&mut self, v: Vec<i32>);
    fn write_chunk_section(&mut self, v: ChunkSection);
    fn write_float(&mut self, v: f32);
    fn write_double(&mut self, v: f64);
    fn write_byte(&mut self, v: i8);
    fn write_u_byte(&mut self, v: u8);
    fn write_boolean(&mut self, v: bool);
    fn write_position(&mut self, v: BlockPosition);
}

impl<T: Read> MinecraftProtocolReader for T {
    fn read_long(&mut self) -> i64 {
        self.read_i64::<BigEndian>().unwrap()
    }

    fn read_var_int(&mut self) -> i32 {
        self.try_read_var_int().unwrap()
    }

    fn try_read_var_int(&mut self) -> Result<i32, Error> {
        read_var_int(self)
    }

    fn read_unsigned_short(&mut self) -> u16 {
        self.read_u16::<BigEndian>().unwrap()
    }

    fn read_unsigned_long(&mut self) -> u64 {
        self.read_u64::<BigEndian>().unwrap()
    }

    fn read_short(&mut self) -> i16 {
        self.read_i16::<BigEndian>().unwrap()
    }

    fn read_string(&mut self) -> String {
        let size = self.read_var_int();

        let mut buffer = vec![0; size as usize];
        self.read_exact(&mut buffer)
            .expect("didn't find enough characters");
        String::from_utf8(buffer).unwrap()
    }

    fn read_u_128(&mut self) -> u128 {
        self.read_u128::<BigEndian>().unwrap()
    }

    fn read_int(&mut self) -> i32 {
        self.read_i32::<BigEndian>().unwrap()
    }

    fn read_int_array(&mut self, length: u32) -> Vec<i32> {
        let mut v = Vec::<i32>::new();
        for _ in 0..length {
            v.push(self.read_i32::<BigEndian>().unwrap());
        }
        v
    }

    fn read_var_int_array(&mut self, length: u32) -> Vec<i32> {
        let mut v = Vec::<i32>::new();
        for _ in 0..length {
            v.push(self.read_var_int());
        }
        v
    }

    fn read_float(&mut self) -> f32 {
        self.read_f32::<BigEndian>().unwrap()
    }

    fn read_chunk_section(&mut self) -> ChunkSection {
        read_chunk_section(self)
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
                panic!("Error while unwrapping boolean");
            }
        }
    }

    fn read_position(&mut self) -> BlockPosition {
        read_position(self)
    }
}

impl<T: Write> MinecraftProtocolWriter for T {
    fn write_long(&mut self, v: i64) {
        self.write_i64::<BigEndian>(v).unwrap();
    }

    fn write_var_int(&mut self, v: i32) {
        write_var_int(self, v)
    }

    fn write_unsigned_short(&mut self, v: u16) {
        self.write_u16::<BigEndian>(v).unwrap()
    }

    fn write_unsigned_long(&mut self, v: u64) {
        self.write_u64::<BigEndian>(v).unwrap();
    }

    fn write_short(&mut self, v: i16) {
        self.write_i16::<BigEndian>(v).unwrap()
    }

    fn write_string(&mut self, v: String) {
        let string_bytes = v.into_bytes();
        self.write_var_int(string_bytes.clone().into_iter().count() as i32);
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

    fn write_var_int_array(&mut self, v: Vec<i32>) {
        v.iter().for_each(|element| self.write_var_int(*element));
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

    fn write_position(&mut self, v: BlockPosition) {
        write_position(self, v);
    }
}

fn read_var_int<S: Read>(stream: &mut S) -> Result<i32, Error> {
    let mut num_read = 0;
    let mut result: i32 = 0;

    loop {
        let value = i32::from(stream.read_u8()?);
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

fn write_var_int<S: Write>(stream: &mut S, v: i32) {
    if v < 0 {
        panic!("Tried to write negative VarInt {:?}", v);
    }
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

//We now have a functional though messy implementation of writing any block
fn write_chunk_section<S: Write>(stream: &mut S, v: ChunkSection) {
    stream.write_u_byte(v.bits_per_block);
    stream.write_var_int(v.data_array_length);
    let mut long: i64 = 0;
    for i in 0..4096 {
        let block_to_place = i64::from(v.block_ids[i as usize]);
        let offset = (PALETTE_SIZE * i) % 64;
        long += block_to_place << offset;
        if ((i * PALETTE_SIZE) % 64) >= 64 - PALETTE_SIZE {
            stream.write_long(long);
            long = block_to_place >> (64 - offset);
        }
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

fn read_chunk_section<S: Read>(stream: &mut S) -> ChunkSection {
    let bits_per_block = stream.read_u_byte();
    if bits_per_block != PALETTE_SIZE as u8 {
        panic!("Cannot read palettes");
    }
    let data_array_length = stream.read_var_int();
    if data_array_length != 896 {
        panic!("Got unexpected data array length");
    }
    let mut block_ids = Vec::<i32>::new();
    let mut long = stream.read_u64::<BigEndian>().unwrap();
    let mut index = 0;
    for i in 0..4096 {
        let bits_to_read = min(64 - (index % 64), 14);
        let left_shift = max(64 - (bits_to_read + (index % 64)), 0);
        let right_shift = left_shift + (index % 64);
        let mut block_id = (long << left_shift) >> right_shift;
        if left_shift == 0 && i != 4095 {
            long = stream.read_u64::<BigEndian>().unwrap();
        }
        if bits_to_read < 14 {
            let remainder_to_read = 14 - bits_to_read;
            let remainder = long << (64 - remainder_to_read) >> (64 - remainder_to_read);
            block_id += remainder << bits_to_read;
        }
        block_ids.push(block_id as i32);
        index += 14;
    }
    //Still ignoring these values for now
    for _ in 0..2048 {
        stream.read_u8().unwrap();
    }
    for _ in 0..2048 {
        stream.read_u8().unwrap();
    }
    ChunkSection {
        bits_per_block,
        data_array_length,
        block_ids,
        block_light: Vec::<u64>::new(),
        sky_light: Vec::<u64>::new(),
    }
}

fn write_position<S: Write>(stream: &mut S, v: BlockPosition) {
    println!(
        "Write position: decoded is (x={:?},y={:?},z={:?})\n",
        v.x, v.y, v.z
    );
    let mut encoded_position: u64 = u64::from(
        ((v.x & 0x03FF_FFFF) << 38) | ((v.z & 0x03FF_FFFF) << 12) | (u32::from(v.y) & 0xFFF),
    );
    println!("Write position: encoded is {:?}", encoded_position);
    stream.write_unsigned_long(encoded_position);
}

fn read_position<S: Read>(stream: &mut S) -> BlockPosition {
    let encoded_position = stream.read_unsigned_long();
    let x = encoded_position >> 38;
    let y = (encoded_position & 0x0000003FFc000000) >> 26;
    let z = (encoded_position & 0x0000000003FFFFFF);
    BlockPosition {
        x: (x as u32),
        y: (y as u16),
        z: (z as u32),
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_section_reading() {
        //Fill block ids with arbitrary pattern
        let mut block_ids = Vec::new();
        for _ in 0..256 {
            block_ids.push(108)
        }
        for y in 1..16 {
            for z in 0..16 {
                for x in 0..16 {
                    block_ids.push(x + y + z)
                }
            }
        }

        let chunk_section = ChunkSection {
            bits_per_block: 14,
            data_array_length: 896,
            block_ids,
            block_light: Vec::new(),
            sky_light: Vec::new(),
        };

        let mut stream = Vec::<u8>::new();
        stream.write_chunk_section(chunk_section.clone());

        // TODO compare this to an expected chunk section (too big to copy paste here)

        let mut stream = std::io::Cursor::new(stream);
        assert_eq!(chunk_section, stream.read_chunk_section());
    }

    #[test]
    fn test_write_var_int() {
        //0
        let mut stream = Vec::<u8>::new();
        stream.write_var_int(0);
        assert_eq!(vec![0], stream);

        //7 bit max
        stream.clear();
        stream.write_var_int(0b1111111);
        assert_eq!(vec![0b1111111], stream);

        //8 bits
        stream.clear();
        stream.write_var_int(0b11111111);
        assert_eq!(vec![0b11111111, 0b1], stream);

        //int max
        stream.clear();
        stream.write_var_int(std::i32::MAX);
        assert_eq!(vec![255, 255, 255, 255, 7], stream);
    }

    #[test]
    #[should_panic]
    fn test_write_var_int_negative() {
        //negative
        let mut stream = Vec::<u8>::new();
        stream.write_var_int(std::i32::MIN);
    }

    #[test]
    fn test_read_var_int() {
        //0
        let mut stream = std::io::Cursor::new(vec![0]);
        assert_eq!(0, read_var_int(&mut stream).unwrap());

        //7 bit max
        let mut stream = std::io::Cursor::new(vec![0b1111111]);
        assert_eq!(0b1111111, read_var_int(&mut stream).unwrap());

        //8 bits
        let mut stream = std::io::Cursor::new(vec![0b11111111, 0b1]);
        assert_eq!(0b11111111, read_var_int(&mut stream).unwrap());

        //int max
        let mut stream = std::io::Cursor::new(vec![255, 255, 255, 255, 7]);
        assert_eq!(std::i32::MAX, read_var_int(&mut stream).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_read_var_int_too_big() {
        //int max
        let mut stream = std::io::Cursor::new(vec![255, 255, 255, 255, 255]);
        read_var_int(&mut stream).unwrap();
    }
}

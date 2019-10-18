extern crate byteorder;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Error, Read, Write};

pub trait MinecraftProtocolReader {
    fn read_unsigned_short(&mut self) -> u16;
    fn read_var_int(&mut self) -> u64;
    fn read_long(&mut self) -> i64;
    fn read_string(&mut self) -> String;
}

pub trait MinecraftProtocolWriter {
    fn write_long(&mut self, v: i64);
    fn write_unsigned_short(&mut self, v: u16);
    fn write_var_int(&mut self, v: u64);
    fn write_string(&mut self, v: String);
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

    fn read_string(&mut self) -> String {
        let size = self.read_var_int();

        let mut buffer = vec![0; size as usize];
        self.read_exact(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl<T: Write> MinecraftProtocolWriter for T {
    fn write_long(&mut self, v: i64) {
        self.write_i64::<BigEndian>(v).unwrap();
    }

    fn write_var_int(&mut self, v: u64) {
        write_var_int(self, v)
    }

    fn write_unsigned_short(&mut self, _v: u16) {
        unimplemented!()
    }

    fn write_string(&mut self, v: String) {
        let string_bytes = v.into_bytes();
        self.write_var_int(string_bytes.clone().into_iter().count() as u64);
        self.write_all(&string_bytes).unwrap();
    }
}

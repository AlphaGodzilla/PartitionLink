use std::io::Cursor;

use bytes::Buf;
use uuid::Error;

use crate::command::Command;

// 协议设计
// Magic(8) + HEAD(1) + VERSION(3) + COMMAND(4) + Length(8) + <CONTENT...>

const MAGIC_PREFIX: u8 = 0xff;

#[derive(Debug, Clone)]
pub enum Bit {
    ONE,
    ZERO,
}

impl Bit {
    pub fn val(&self) -> bool {
        match self {
            Bit::ONE => true,
            Bit::ZERO => false,
        }
    }
}

pub struct BitVec(Vec<Bit>);

impl BitVec {
    pub fn to_byte(&self) -> anyhow::Result<u8> {
        let u8_slice = &self.0[0..8];
        to_byte(u8_slice, 0)
    }
}

impl From<u8> for BitVec {
    fn from(value: u8) -> Self {
        BitVec(to_bit_vec(value as u32, 8))
    }
}

pub fn to_bit_vec(value: u32, bit_size: usize) -> Vec<Bit> {
    let mut bits = Vec::with_capacity(bit_size);
    let mut copy = value.clone();
    for _ in 0..bit_size {
        if copy == 0 {
            bits.push(Bit::ZERO);
        }
        if copy & 1 != 0 {
            bits.push(Bit::ONE);
        } else {
            bits.push(Bit::ZERO);
        }
        copy = copy >> 1;
    }
    bits.reverse();
    bits
}

pub fn to_byte(value: &[Bit], from: usize) -> anyhow::Result<u8> {
    let end = from + 8;
    if value.len() < end {
        return Err(anyhow::anyhow!("value slice not enought long"));
    }
    let mut byte: u8 = 0;
    for i in from..8 {
        let bit = &value[i];
        match bit {
            Bit::ONE => {
                byte = byte | 1 << (7 - i);
            }
            _ => {}
        }
    }
    Ok(byte)
}

pub trait Segment {
    // 比特位数
    fn bits() -> usize;
    // 值
    fn value(&self) -> Vec<Bit>;
}

pub enum Head {
    UNFIN,
    FIN,
}
impl Segment for Head {
    fn bits() -> usize {
        4
    }

    fn value(&self) -> Vec<Bit> {
        match self {
            Head::UNFIN => vec![Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ZERO],
            Head::FIN => vec![Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ONE],
        }
    }
}

pub struct Padding(Bit);

impl Segment for Padding {
    fn bits() -> usize {
        1
    }

    fn value(&self) -> Vec<Bit> {
        match self.0 {
            Bit::ZERO => vec![Bit::ZERO],
            Bit::ONE => vec![Bit::ONE],
        }
    }
}

pub struct Version {
    inner: u8,
}

impl Version {
    pub fn new(value: u8) -> anyhow::Result<Self> {
        if value > 15 {
            return Err(anyhow::anyhow!("version can not be grant than 8"));
        }
        Ok(Version { inner: value })
    }
}

impl Segment for Version {
    fn bits() -> usize {
        3
    }

    fn value(&self) -> Vec<Bit> {
        to_bit_vec(self.inner as u32, Self::bits())
    }
}

pub struct Length(u32);

impl Length {
    pub fn new(value: u32) -> Self {
        Length(value)
    }
}

impl Segment for Length {
    fn bits() -> usize {
        size_of::<u32>()
    }

    fn value(&self) -> Vec<Bit> {
        to_bit_vec(self.0, Self::bits())
    }
}
#[derive(Clone, Debug)]
pub enum FrameMatchResult {
    Incomplete,
    MissMatch,
    Complete,
}

pub struct Frame {
    head: Head,
    version: Version,
    cmd: Command,
    length: Length,
    payload: Vec<u8>,
}
impl Frame {
    pub fn new() -> Self {
        Frame {
            head: Head::FIN,
            version: Version::new(1).expect("version constract error"),
            cmd: Command::OP,
            length: Length::new(0),
            payload: Vec::new(),
        }
    }

    pub fn set_head(&mut self, head: Head) -> &Frame {
        self.head = head;
        self
    }

    // pub fn set_version(&mut self, version: Version) -> &Frame {
    //     self.version = version;
    //     self
    // }

    pub fn set_cmd(&mut self, cmd: Command) -> &Frame {
        self.cmd = cmd;
        self
    }

    pub fn set_length(&mut self, length: Length) -> &Frame {
        self.length = length;
        self
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) -> &Frame {
        self.payload = payload;
        self
    }

    pub fn check(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<FrameMatchResult> {
        // check magic
        let has_remain = cursor.has_remaining();
        if !has_remain {
            return Ok(FrameMatchResult::Incomplete);
        }
        let magic_prefix: u8 = cursor.get_u8();
        if magic_prefix != MAGIC_PREFIX {
            return Ok(FrameMatchResult::MissMatch);
        }

        // check header
        let has_remain = cursor.has_remaining();
        if !has_remain {
            return Ok(FrameMatchResult::Incomplete);
        }
        let header = cursor.get_u8();
        let header: BitVec = header.into();
        let header = header.0;

        // chcke version
        let version_vec = &header[1..4];
        let mut version = Vec::with_capacity(Head::bits());
        version.extend_from_slice(version_vec);
        let version: u8 = BitVec(version).to_byte()?;
        if (version != 1) {
            return Ok(FrameMatchResult::MissMatch);
        }

        // checke command
        let command_vec = &header[4..];
        let mut command = Vec::with_capacity(Command::bits());
        command.extend_from_slice(command_vec);
        let command: Command = BitVec(command).to_byte()?.into();
        if command == Command::UNKNOWN {
            return Ok(FrameMatchResult::MissMatch);
        }

        // check length
        let has_remain = cursor.has_remaining();
        if !has_remain {
            return Ok(FrameMatchResult::Incomplete);
        }
        let length: u8 = cursor.get_u8();
        if length > 0 {
            // check payload
            // [1,2,3,4]
            let pos = cursor.position() as usize;
            let len = cursor.get_ref().len();
            let remian_len = len - pos;
            if remian_len != (length as usize) {
                // payload长度不符合
                return Ok(FrameMatchResult::Incomplete);
            }
        }
        Ok(FrameMatchResult::Complete)
    }
}

#[cfg(test)]
mod test {
    use super::{Segment, Version};

    #[test]
    fn version_test() {
        let version = Version::new(5).unwrap();
        println!("{:?}", version.value());
    }

    #[test]
    fn slice_test() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        println!("[..1] = {:?}", &vec[..1]);
        println!("[..1] = {:?}", &vec[1..3]);
    }

    #[test]
    fn iter_test() {
        for i in 0..8 {
            println!("iter_test: {}", i);
        }
    }
}

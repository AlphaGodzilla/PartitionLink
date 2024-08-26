// 协议设计
// Magic(8) + HEAD(1) + VERSION(3) + OPERATOR(4) + Length(8) + <CONTENT...>

use crate::protocol::Bit::ONE;
use crate::protocol::Bit::ZERO;

const MAGIC_PREFIX: u8 = 0xff;

pub mod frame;
pub mod head;
pub mod header;
pub mod length;
pub mod op;
pub mod version;

#[derive(Debug, Clone)]
pub enum Bit {
    ONE,
    ZERO,
}

pub struct BitVec(Vec<Bit>);
impl BitVec {
    pub fn to_byte(&self) -> anyhow::Result<u8> {
        let len = self.0.len();
        if len < 8 {
            // 向量长度
            let mut self_vec_copy = Vec::with_capacity(8);
            for _ in 0..(8 - len) {
                self_vec_copy.push(ZERO);
            }
            self_vec_copy.extend_from_slice(&self.0[..]);
            return to_byte(&self_vec_copy[..], 0);
        }
        let u8_slice = &self.0[(len - 8)..];
        to_byte(u8_slice, 0)
    }

    pub fn to_u8(&self) -> anyhow::Result<u8> {
        self.to_byte()
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
            bits.insert(0, ZERO);
            continue;
        }
        if copy & 1 != 0 {
            bits.insert(0, ONE);
            copy = copy >> 1;
        } else {
            bits.insert(0, ZERO);
            copy = copy >> 1;
        }
    }
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
    // 计算byte
    fn to_byte(&self) -> u8;
}

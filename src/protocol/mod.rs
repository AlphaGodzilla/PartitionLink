/// 协议设计
/// Magic(8) + HEAD(1) + VERSION(3) + OPERATOR(4) + Length(8) + <CONTENT...>

const MAGIC_PREFIX: u8 = 0xff;
const CURRENT_VERSION: u8 = 1;

pub mod frame;
pub mod head;
pub mod header;
pub mod length;
pub mod op;
pub mod version;

pub trait Segment {
    // 比特位数
    fn bits() -> usize;
    // 计算byte
    fn to_byte(&self) -> u8;
    // 从byte解析
    fn from_byte(byte: u8) -> Self;
}

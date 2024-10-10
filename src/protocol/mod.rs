/// 协议设计
/// Magic(8) + HEAD(1) + VERSION(3) + KIND(4) + Length(8) + <CONTENT...>

// 截帧魔法值
pub const MAGIC_PREFIX: u8 = 0xff;
// 当前协议版本
pub const CURRENT_VERSION: u8 = 1;
// payload最大长度
pub const MAX_PAYLOAD_LENGTH: u8 = 255;

pub mod frame;
pub mod head;
pub mod header;
pub mod kind;
pub mod length;
pub mod version;

pub trait Segment {
    // 比特位数
    fn bits() -> usize;
    // 计算byte
    fn to_byte(&self) -> u8;
    // 从byte解析
    fn from_byte(byte: u8) -> Self;
}

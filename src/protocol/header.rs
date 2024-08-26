use super::{head::Head, length::Length, op::Operator, version::Version, Segment};

#[derive(Debug)]
pub struct Header {
    head: Head,
    version: Version,
    op: Operator,
}

impl Segment for Header {
    fn bits() -> usize {
        8
    }

    fn value(&self) -> Vec<super::Bit> {
        Vec::new()
    }

    fn to_byte(&self) -> u8 {
        let mut byte: u8 = 0;
        let mut count = 0;
        byte |= self.head.to_byte() << (8 - Head::bits() - count);
        count += Head::bits();
        byte |= self.version.to_bytes() << (8 - Version::bits() - count);
        count += Version::bits();
        byte
    }
}

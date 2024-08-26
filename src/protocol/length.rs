use super::{to_bit_vec, Bit, Segment};
use crate::proto::Bit::ONE;
use crate::proto::Bit::ZERO;

#[derive(Debug)]
pub struct Length(u8);

impl Length {
    pub fn new(value: u8) -> Self {
        Length(value)
    }

    pub fn inner_value(&self) -> u8 {
        self.0
    }
}

impl Segment for Length {
    fn bits() -> usize {
        size_of::<u8>()
    }

    fn value(&self) -> Vec<Bit> {
        to_bit_vec(self.0 as u32, Self::bits())
    }
}

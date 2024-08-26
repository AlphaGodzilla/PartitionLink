use super::Segment;

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

    fn to_byte(&self) -> u8 {
        self.0
    }

    fn from_byte(byte: u8) -> Self {
        Self::new(byte)
    }
}

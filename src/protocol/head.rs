use super::Segment;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Head {
    UNFIN,
    FIN,
}
impl Segment for Head {
    fn bits() -> usize {
        1
    }

    fn to_byte(&self) -> u8 {
        match self {
            Head::UNFIN => 0b0,
            Head::FIN => 0b0000_0001,
        }
    }

    fn from_byte(byte: u8) -> Self {
        match byte {
            0b0 => Self::UNFIN,
            0b0000_0001 => Self::FIN,
            _ => Self::FIN,
        }
    }
}

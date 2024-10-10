use super::Segment;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Kind {
    UNKNOWN,
    PING,
    PONG,
    CMD,
    RAFT,
}

impl From<u8> for Kind {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::PING,
            1 => Self::PONG,
            2 => Self::CMD,

            _ => Self::UNKNOWN,
        }
    }
}

impl Segment for Kind {
    fn bits() -> usize {
        4
    }

    fn to_byte(&self) -> u8 {
        match self {
            Self::PING => 0b0,
            Self::PONG => 0b0000_0001,
            Self::CMD => 0b0000_0010,
            Self::RAFT => 0b0000_0011,
            Self::UNKNOWN => 0b0000_1111,
        }
    }

    fn from_byte(byte: u8) -> Self {
        match byte {
            0b0 => Self::PING,
            0b0000_0001 => Self::PONG,
            0b0000_0010 => Self::CMD,
            _ => Self::UNKNOWN,
        }
    }
}

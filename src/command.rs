use crate::protocol::Bit::ONE;
use crate::protocol::Bit::ZERO;
use crate::protocol::{Bit, Segment};

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Command {
    UNKNOWN,
    PING,
    PONG,
    OP,
}

impl From<u8> for Command {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::PING,
            1 => Self::PONG,
            3 => Self::OP,
            _ => Self::UNKNOWN,
        }
    }
}

impl Segment for Command {
    fn bits() -> usize {
        4
    }

    fn value(&self) -> Vec<crate::protocol::Bit> {
        match self {
            Self::PING => vec![ZERO, ZERO, ZERO, ZERO],
            Self::PONG => vec![ZERO, ZERO, ZERO, ONE],
            Self::OP => vec![ZERO, ZERO, ONE, ZERO],
            Self::UNKNOWN => vec![],
        }
    }
}

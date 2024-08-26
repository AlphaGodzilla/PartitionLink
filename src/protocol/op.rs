use crate::protocol::Bit::ONE;
use crate::protocol::Bit::ZERO;

use super::Bit;
use super::Segment;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Operator {
    UNKNOWN,
    PING,
    PONG,
    OP,
}

impl From<u8> for Operator {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::PING,
            1 => Self::PONG,
            2 => Self::OP,
            _ => Self::UNKNOWN,
        }
    }
}

impl Segment for Operator {
    fn bits() -> usize {
        4
    }

    fn value(&self) -> Vec<Bit> {
        match self {
            Self::PING => vec![ZERO, ZERO, ZERO, ZERO],
            Self::PONG => vec![ZERO, ZERO, ZERO, ONE],
            Self::OP => vec![ZERO, ZERO, ONE, ZERO],
            Self::UNKNOWN => vec![],
        }
    }
}

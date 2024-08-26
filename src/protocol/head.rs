use super::{Bit, Segment};
use crate::proto::Bit::ONE;
use crate::proto::Bit::ZERO;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Head {
    UNFIN,
    FIN,
}
impl Segment for Head {
    fn bits() -> usize {
        4
    }

    fn value(&self) -> Vec<Bit> {
        match self {
            Head::UNFIN => vec![Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ZERO],
            Head::FIN => vec![Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ONE],
        }
    }
}
impl From<Bit> for Head {
    fn from(value: Bit) -> Self {
        match value {
            ZERO => Head::UNFIN,
            ONE => Head::FIN,
        }
    }
}

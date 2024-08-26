use super::to_bit_vec;
use super::Bit;
use super::Segment;

#[derive(Debug)]
pub struct Version {
    inner: u8,
}
impl Version {
    pub fn new(value: u8) -> anyhow::Result<Self> {
        if value > 15 {
            return Err(anyhow::anyhow!("version can not be grant than 8"));
        }
        Ok(Version { inner: value })
    }
}
impl Segment for Version {
    fn bits() -> usize {
        3
    }

    fn value(&self) -> Vec<Bit> {
        to_bit_vec(self.inner as u32, Self::bits())
    }
}

use super::Segment;

#[derive(Debug)]
pub struct Version {
    inner: u8,
}
impl Version {
    pub fn new(value: u8) -> anyhow::Result<Self> {
        if value > 8 {
            return Err(anyhow::anyhow!("version can not be grant than 8"));
        }
        Ok(Version { inner: value })
    }
}
impl Segment for Version {
    fn bits() -> usize {
        3
    }

    fn to_byte(&self) -> u8 {
        self.inner
    }

    fn from_byte(byte: u8) -> Self {
        Self::new(byte).unwrap()
    }
}

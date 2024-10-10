use log::trace;

use super::{head::Head, kind::Kind, version::Version, Segment};

#[derive(Debug)]
pub struct Header {
    pub head: Head,
    pub version: Version,
    pub kind: Kind,
}

impl Segment for Header {
    fn bits() -> usize {
        8
    }

    fn to_byte(&self) -> u8 {
        let mut byte: u8 = 0;
        let mut count = 0;
        byte |= self.head.to_byte() << (8 - Head::bits() - count);
        trace!("head left move {}", 8 - Head::bits() - count);
        count += Head::bits();
        byte |= self.version.to_byte() << (8 - Version::bits() - count);
        trace!("version left move {}", 8 - Version::bits() - count);
        count += Version::bits();
        byte |= self.kind.to_byte() << (8 - Kind::bits()) >> (8 - Kind::bits())
            << (8 - Kind::bits() - count);
        trace!("op left move {}", 8 - Kind::bits() - count);
        // println!("op phase: {}", byte);
        byte
    }

    fn from_byte(byte: u8) -> Self {
        Header {
            head: Head::from_byte(byte >> (8 - Head::bits())),
            version: Version::new(byte << Head::bits() >> (8 - Version::bits())).unwrap(),
            kind: Kind::from_byte(byte << Head::bits() << Version::bits() >> (8 - Kind::bits())),
        }
    }
}

use std::io::Cursor;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::trace;

use super::{
    head::Head, length::Length, op::Operator, version::Version, BitVec, Segment, MAGIC_PREFIX,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameMatchResult<'a> {
    Incomplete(&'a str),
    MissMatch(&'a str),
    Complete,
}

#[derive(Debug)]
pub struct Frame {
    pub head: Head,
    pub version: Version,
    pub op: Operator,
    pub length: Length,
    pub payload: Vec<u8>,
    encode_raw: Option<Bytes>,
}
impl Frame {
    pub fn new() -> Self {
        Frame {
            head: Head::FIN,
            version: Version::new(1).expect("version constract error"),
            op: Operator::OP,
            length: Length::new(0),
            payload: Vec::new(),
            encode_raw: None,
        }
    }

    pub fn set_head(&mut self, head: Head) -> &mut Frame {
        self.head = head;
        self
    }

    // pub fn set_version(&mut self, version: Version) -> &Frame {
    //     self.version = version;
    //     self
    // }

    pub fn set_op(&mut self, cmd: Operator) -> &mut Frame {
        self.op = cmd;
        self
    }

    pub fn set_length(&mut self, length: Length) -> &mut Frame {
        self.length = length;
        self
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) -> &mut Frame {
        self.payload = payload;
        self
    }

    pub fn encode(&mut self) -> &[u8] {
        if self.encode_raw.is_some() {
            return self.encode_raw.as_ref().unwrap();
        }
        let mut buff = BytesMut::new();
        buff.put_u8(MAGIC_PREFIX);
        let mut header = Vec::with_capacity(8);
        header.extend_from_slice(&self.head.value()[..]);
        header.extend_from_slice(&self.version.value()[..]);
        header.extend_from_slice(&self.op.value()[..]);
        let header = BitVec(header).to_byte().unwrap();
        buff.put_u8(header);
        buff.put_u8(self.length.inner_value());
        self.payload.iter().for_each(|b| buff.put_u8(*b));
        self.encode_raw = Some(buff.freeze());
        self.encode_raw.as_ref().unwrap()
    }

    pub fn parse(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Frame> {
        let mut frame = Frame::new();
        // skip magic
        cursor.get_u8();

        let header = cursor.get_u8();
        let header: BitVec = header.into();
        let header = header.0;
        let head = header.get(0).unwrap();
        frame.set_head(head.clone().into());

        let command_vec = &header[4..];
        let mut command = Vec::with_capacity(Operator::bits());
        command.extend_from_slice(command_vec);
        let command: Operator = BitVec(command).to_byte()?.into();
        frame.set_op(command);

        let length: u8 = cursor.get_u8();
        frame.set_length(Length::new(length));

        let mut payload = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let byte = cursor.get_u8();
            payload.push(byte);
        }
        frame.set_payload(payload);

        Ok(frame)
    }

    pub fn check<'a>(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<FrameMatchResult<'a>> {
        // check magic
        let has_remain = cursor.has_remaining();
        if !has_remain {
            return Ok(FrameMatchResult::Incomplete("no_data"));
        }
        let magic_prefix: u8 = cursor.get_u8();
        if magic_prefix != MAGIC_PREFIX {
            trace!("magic, {}", magic_prefix);
            return Ok(FrameMatchResult::MissMatch("magic"));
        }

        // check header
        let has_remain = cursor.has_remaining();
        if !has_remain {
            return Ok(FrameMatchResult::Incomplete("header"));
        }
        let header = cursor.get_u8();
        let header: BitVec = header.into();
        let header = header.0;

        // chcke version
        let version_vec = &header[1..4];
        let mut version = Vec::with_capacity(Head::bits());
        version.extend_from_slice(version_vec);
        let version: u8 = BitVec(version).to_byte()?;
        if version != 1 {
            return Ok(FrameMatchResult::MissMatch("version"));
        }

        // checke command
        let command_vec = &header[4..];
        let mut command = Vec::with_capacity(Operator::bits());
        command.extend_from_slice(command_vec);
        let command: Operator = BitVec(command).to_byte()?.into();
        if command == Operator::UNKNOWN {
            return Ok(FrameMatchResult::MissMatch("op"));
        }

        // check length
        let has_remain = cursor.has_remaining();
        if !has_remain {
            return Ok(FrameMatchResult::Incomplete("length"));
        }
        let length: u8 = cursor.get_u8();
        if length > 0 {
            // check payload
            // [1,2,3,4]
            let pos = cursor.position() as usize;
            let len = cursor.get_ref().len();
            let remian_len = len - pos;
            if remian_len != (length as usize) {
                // payload长度不符合
                return Ok(FrameMatchResult::MissMatch("length"));
            }
        }
        Ok(FrameMatchResult::Complete)
    }

    pub fn is_last(&self) -> bool {
        self.head == Head::FIN
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use bytes::{BufMut, BytesMut};

    use crate::protocol::{
        frame::{Frame, FrameMatchResult},
        head::Head,
        op::Operator,
        version::Version,
        BitVec, Segment, MAGIC_PREFIX,
    };

    #[test]
    fn version_test() {
        let version = Version::new(5).unwrap();
        println!("{:?}", version.value());
    }

    #[test]
    fn slice_test() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        println!("[..1] = {:?}", &vec[..1]);
        println!("[..1] = {:?}", &vec[1..3]);
    }

    #[test]
    fn iter_test() {
        let size = 8;
        for i in 0..size {
            println!("iter_test: {}", i);
        }
    }

    fn build_complete_bytes_buff(
        mistake_magic: bool,
        mistake_version: bool,
        mistake_op: bool,
        mistake_payload_length: bool,
    ) -> BytesMut {
        let mut buff = bytes::BytesMut::new();
        // 写入数据
        // magic
        if mistake_magic {
            buff.put_u8(0xf1);
        } else {
            buff.put_u8(MAGIC_PREFIX);
        }

        // head
        let head = Head::FIN.value();
        // version
        let version;
        if mistake_version {
            version = Version::new(2).unwrap().value();
        } else {
            version = Version::new(1).unwrap().value();
        }
        // op
        let op;
        if mistake_op {
            op = Operator::UNKNOWN.value();
        } else {
            op = Operator::OP.value();
        }

        let mut header = Vec::new();
        header.extend_from_slice(&head[..]);
        header.extend_from_slice(&version[..]);
        header.extend_from_slice(&op[..]);
        let header = BitVec(header).to_byte().unwrap();
        buff.put_u8(header);
        // length
        if mistake_payload_length {
            buff.put_u8(10);
        } else {
            buff.put_u8(1);
        }
        // payload
        buff.put_u8(0xff);
        buff
    }

    #[test]
    fn mismatch_frame_check_for_magic_test() {
        let mistake_magic_buff = build_complete_bytes_buff(true, false, false, false);
        let mut cursor = Cursor::new(&mistake_magic_buff[..]);
        assert_eq!(
            Frame::check(&mut cursor).unwrap(),
            FrameMatchResult::MissMatch("magic")
        );
    }

    #[test]
    fn mismatch_frame_check_for_version_test() {
        let buff = build_complete_bytes_buff(false, true, false, false);
        let mut cursor = Cursor::new(&buff[..]);
        assert_eq!(
            Frame::check(&mut cursor).unwrap(),
            FrameMatchResult::MissMatch("version")
        );
    }

    #[test]
    fn mismatch_frame_check_for_op_test() {
        let buff = build_complete_bytes_buff(false, false, true, false);
        let mut cursor = Cursor::new(&buff[..]);
        assert_eq!(
            Frame::check(&mut cursor).unwrap(),
            FrameMatchResult::MissMatch("op")
        );
    }

    #[test]
    fn incomplete_frame_check_for_empty_buffer_test() {
        let mut buff = bytes::BytesMut::new();
        let mut cursor = Cursor::new(&buff[..]);
        assert_eq!(
            Frame::check(&mut cursor).unwrap(),
            FrameMatchResult::Incomplete("no_data")
        );
    }

    #[test]
    fn incomplete_frame_check_for_length_test() {
        let buff = build_complete_bytes_buff(false, false, false, true);
        let mut cursor = Cursor::new(&buff[..]);
        assert_eq!(
            Frame::check(&mut cursor).unwrap(),
            FrameMatchResult::MissMatch("length")
        );
    }

    #[test]
    fn complete_frame_check_test() {
        let buff = build_complete_bytes_buff(false, false, false, false);
        let mut cursor = Cursor::new(&buff[..]);
        assert_eq!(
            Frame::check(&mut cursor).unwrap(),
            FrameMatchResult::Complete
        );
    }
}

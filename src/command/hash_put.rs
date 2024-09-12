use std::fmt::Display;

use ahash::AHashMap;
use prost::Message;

use crate::db::{database::Database, dbvalue::DBValue};

use super::{
    proto::{ProtoCmd, ProtoCommand, ProtoDbValue, ProtoHashMapPutCmd},
    ExecutableCommand,
};
use anyhow::anyhow;

#[derive(Clone)]
pub struct HashMapPutCmd {
    pub key: String,
    pub member_key: String,
    pub member_value: DBValue,
}

impl ExecutableCommand for HashMapPutCmd {
    fn cmd_type(&self) -> super::CommandType {
        super::CommandType::WRITE
    }

    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        match db.get_mut(&self.key) {
            Some(dbvalue) => match dbvalue {
                DBValue::Hash(hash) => {
                    hash.insert(self.member_key.clone(), self.member_value.clone());
                    return Ok(None);
                }
                _ => {
                    return Err(anyhow!(
                        "Mismatch DBValue type, required Hash but got {}",
                        dbvalue
                    ));
                }
            },
            None => {
                let mut hashmap = AHashMap::new();
                hashmap.insert(self.member_key.clone(), self.member_value.clone());
                db.set(self.key.clone(), DBValue::Hash(hashmap));
                return Ok(None);
            }
        }
    }

    fn encode(&self) -> anyhow::Result<bytes::Bytes> {
        let mut buff = bytes::BytesMut::new();
        let msg = ProtoHashMapPutCmd {
            key: self.key.clone(),
            member_key: self.member_key.clone(),
            member_value: Some(ProtoDbValue {
                value: Some(self.member_value.to_protobuf().into()),
            }),
        };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    fn cmd_id(&self) -> i32 {
        ProtoCmd::HashMapPutCmd as i32
    }
}
impl Display for HashMapPutCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashPut")
    }
}

impl From<ProtoHashMapPutCmd> for HashMapPutCmd {
    fn from(value: ProtoHashMapPutCmd) -> Self {
        HashMapPutCmd {
            key: value.key,
            member_key: value.member_key,
            member_value: value.member_value.map_or(DBValue::None, |x| x.into()),
        }
    }
}

impl TryFrom<ProtoCommand> for HashMapPutCmd {
    type Error = anyhow::Error;

    fn try_from(value: ProtoCommand) -> Result<Self, Self::Error> {
        let cmd = ProtoHashMapPutCmd::decode(&value.value[..])?;
        Ok(cmd.into())
    }
}

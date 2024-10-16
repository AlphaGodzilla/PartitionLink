use std::any::Any;
use std::fmt::Display;

use ahash::AHashMap;
use prost::Message;

use crate::db::{database::Database, dbvalue::DBValue};

use super::{
    proto::{ProtoCmd, ProtoCommand, ProtoDbValue, ProtoHashMapPutCmd},
    ExecutableCommand,
};
use anyhow::anyhow;
use async_trait::async_trait;
use crate::runtime::Runtime;

#[derive(Clone)]
pub struct HashMapPutCmd {
    pub key: String,
    pub member_key: String,
    pub member_value: DBValue,
}

#[async_trait]
impl ExecutableCommand for HashMapPutCmd {
    fn cmd_type(&self) -> super::CommandType {
        super::CommandType::WRITE
    }

    async fn execute(&self, app: Option<&Runtime>, db: Option<&mut Database>) -> anyhow::Result<Option<DBValue>> {
        if let Some(db) = db {
            return match db.get_mut(&self.key) {
                Some(value) => match value {
                    DBValue::Hash(ref mut hash) => {
                        hash.insert(self.member_key.clone(), self.member_value.clone());
                        Ok(None)
                    }
                    _ => {
                        Err(anyhow!("Mismatch DBValue type, required Hash but got {}",value))
                    }
                },
                None => {
                    let mut hashmap = AHashMap::new();
                    hashmap.insert(self.member_key.clone(), self.member_value.clone());
                    db.set(self.key.clone(), DBValue::Hash(hashmap));
                    Ok(None)
                }
            };
        }
        Ok(None)
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

    fn cmd_id(&self) -> ProtoCmd {
        ProtoCmd::HashMapPutCmd
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Display for HashMapPutCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashPut {} {}", &self.key, &self.member_value)
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

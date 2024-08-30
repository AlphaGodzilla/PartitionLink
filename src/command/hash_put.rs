use std::{collections::HashMap, fmt::Display};

use ahash::AHashMap;
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::db::dbvalue::DBValue;

use super::ExecutableCommand;
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

    fn execute(&self, db: &mut crate::db::Database) -> anyhow::Result<Option<DBValue>> {
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
        let msg = super::proto::out::HashMapPutCmd {
            key: self.key.clone(),
            member_key: self.member_key.clone(),
            member_value: Some(super::proto::out::DbValue {
                value: Some(self.member_value.to_protobuf().into()),
            }),
        };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }
}
impl Display for HashMapPutCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashPut")
    }
}

impl From<super::proto::out::HashMapPutCmd> for HashMapPutCmd {
    fn from(value: super::proto::out::HashMapPutCmd) -> Self {
        HashMapPutCmd {
            key: value.key,
            member_key: value.member_key,
            member_value: value.member_value.map_or(DBValue::None, |x| x.into()),
        }
    }
}

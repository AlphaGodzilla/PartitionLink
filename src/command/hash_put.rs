use std::any::Any;
use std::fmt::Display;

use ahash::AHashMap;

use crate::db::{database::Database, dbvalue::DBValue};

use super::ExecutableCommand;
use crate::proto::command_message::Cmd;
use crate::proto::HashPutCmd;
use crate::runtime::Runtime;
use anyhow::anyhow;
use async_trait::async_trait;
// #[derive(Clone)]
// pub struct HashPutCmd {
//     pub key: String,
//     pub member_key: String,
//     pub member_value: DBValue,
// }

#[async_trait]
impl ExecutableCommand for HashPutCmd {
    fn cmd_type(&self) -> super::CommandType {
        super::CommandType::WRITE
    }

    async fn execute(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<Option<DBValue>> {
        if let Some(db) = db {
            return match db.get_mut(&self.key) {
                Some(value) => match value {
                    DBValue::Hash(ref mut hash) => {
                        if let Some(member_value) = &self.member_value {
                            hash.insert(self.member_key.clone(), member_value.clone().into());
                        }
                        Ok(None)
                    }
                    _ => Err(anyhow!(
                        "Mismatch DBValue type, required Hash but got {}",
                        value
                    )),
                },
                None => {
                    if let Some(member_value) = &self.member_value {
                        let mut hashmap = AHashMap::new();
                        hashmap.insert(self.member_key.clone(), member_value.clone().into());
                        db.set(self.key.clone(), DBValue::Hash(hashmap));
                    }
                    Ok(None)
                }
            };
        }
        Ok(None)
    }

    fn to_cmd(&self) -> anyhow::Result<Cmd> {
        Ok(Cmd::HashPut(self.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Display for HashPutCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(member_value) = &self.member_value {
            let value: DBValue = member_value.clone().into();
            write!(f, "HashPut {} {}", &self.key, &value)
        } else {
            write!(f, "HashPut {} None", &self.key)
        }
    }
}

impl TryFrom<Cmd> for HashPutCmd {
    type Error = anyhow::Error;

    fn try_from(value: Cmd) -> Result<Self, Self::Error> {
        if let Cmd::HashPut(hash) = value {
            Ok(hash)
        } else {
            Err(anyhow::anyhow!("invalid command"))
        }
    }
}

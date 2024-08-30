use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::db::DBValue;

use super::ExecutableCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
                let mut hashmap = HashMap::new();
                hashmap.insert(self.member_key.clone(), self.member_value.clone());
                db.set(self.key.clone(), DBValue::Hash(hashmap));
                return Ok(None);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashMapGetCmd {
    pub key: String,
    pub member_key: String,
}

impl ExecutableCommand for HashMapGetCmd {
    fn cmd_type(&self) -> super::CommandType {
        super::CommandType::READ
    }

    fn execute(&self, db: &mut crate::db::Database) -> anyhow::Result<Option<DBValue>> {
        if let Some(value) = db.get(&self.key) {
            match value {
                DBValue::Hash(hash) => {
                    return Ok(hash.get(&self.member_key).map(|x| x.clone()));
                }
                _ => return Ok(None),
            }
        }
        return Ok(None);
    }
}

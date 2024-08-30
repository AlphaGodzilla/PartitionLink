use crate::db::{DBValue, Database};

use super::{CommandType, ExecutableCommand};

#[derive(Debug, Clone)]
pub struct HelloCmd {
    pub valid: bool,
}

impl ExecutableCommand for HelloCmd {
    fn cmd_type(&self) -> CommandType {
        CommandType::READ
    }

    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        Ok(None)
    }
}

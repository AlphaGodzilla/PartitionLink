use super::ExecutableCommand;
use crate::proto::command_message::Cmd;

pub fn parse_proto_command(cmd: Cmd) -> anyhow::Result<Box<dyn ExecutableCommand>> {
    match cmd {
        Cmd::Hello(v) => Ok(Box::new(v)),
        Cmd::HashPut(v) => Ok(Box::new(v)),
        Cmd::HashGet(v) => Ok(Box::new(v)),
        Cmd::Raft(v) => Ok(Box::new(v)),
    }
}

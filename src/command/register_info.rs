use super::{
    hash_get::HashMapGetCmd,
    hash_put::HashMapPutCmd,
    hello::HelloCmd,
    invalid::InvalidCommand,
    proto::{ProtoCmd, ProtoCommand},
    ExecutableCommand,
};

pub fn parse_proto_command(cmd: ProtoCommand) -> anyhow::Result<Box<dyn ExecutableCommand>> {
    let p_cmd = ProtoCmd::try_from(cmd.cmd)?;
    match p_cmd {
        ProtoCmd::HelloCmd => Ok(Box::new(HelloCmd::try_from(cmd)?)),
        ProtoCmd::HashMapPutCmd => Ok(Box::new(HashMapPutCmd::try_from(cmd)?)),
        ProtoCmd::HashMapGetCmd => Ok(Box::new(HashMapGetCmd::try_from(cmd)?)),
        _ => Ok(Box::new(InvalidCommand {})),
    }
}

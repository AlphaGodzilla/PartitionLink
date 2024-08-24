#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CMD {
    UNKNOWN,
    HELLO,
}
impl From<&str> for CMD {
    fn from(value: &str) -> Self {
        match value {
            "hello" => CMD::HELLO,
            _ => CMD::UNKNOWN,
        }
    }
}

#[derive(Debug)]
pub struct Command {
    pub cmd: CMD,
}

impl Command {}

impl From<Vec<u8>> for Command {
    fn from(value: Vec<u8>) -> Self {
        // todo!()
        let cmd = &(String::from_utf8_lossy(&value).to_string())[..];
        let cmd: CMD = cmd.into();
        Command { cmd }
    }
}

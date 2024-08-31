pub mod out {
    include!(concat!(env!("OUT_DIR"), "/command.cmd.rs"));
    include!(concat!(env!("OUT_DIR"), "/command.rs"));
}

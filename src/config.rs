use std::time::Duration;

pub struct Config {
    pub disc_multicast_group: String,
    pub disc_multicast_port: usize,
    pub disc_multicast_interval: Duration,
    // 节点存活有效时间
    pub disc_multicast_ttl: Duration,
    // 节点存活过期检查任务执行间隔
    pub disc_multicast_ttl_check_interval: Duration,
    // 命令服务器监听端口
    pub listen_port: usize,
    // 命令服务器监听地址
    pub listen_addr: String,

    // raft配置
    pub raft_config: raft::prelude::Config,
    pub raft_loop_interval: Duration,
}

impl Config {
    pub fn new() -> Self {
        let listen_port = option_env!("PL_LISTEN_PORT");
        Config {
            disc_multicast_group: String::from("224.0.0.1"),
            disc_multicast_port: 54123,
            disc_multicast_interval: Duration::from_secs(10),
            disc_multicast_ttl: Duration::from_secs(30),
            disc_multicast_ttl_check_interval: Duration::from_secs(10),
            listen_port: listen_port.map_or(7111, |port| usize::from_str_radix(port, 10).unwrap()),
            listen_addr: String::from("0.0.0.0"),
            raft_config: raft::prelude::Config {
                election_tick: 10,
                heartbeat_tick: 3,
                ..Default::default()
            },
            raft_loop_interval: Duration::from_secs(1),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

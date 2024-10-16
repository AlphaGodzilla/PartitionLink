use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct Config {
    // 节点ID
    pub node_id: u64,
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
        let mut cfg = Config {
            node_id: 0,
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
        };
        let node_id = Uuid::new_v4().to_string();
        let mut hasher = DefaultHasher::new();
        node_id.hash(&mut hasher);
        let node_id = hasher.finish();
        cfg.node_id = node_id;
        cfg.raft_config.id = node_id;
        cfg
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

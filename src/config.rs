use std::time::Duration;

pub struct Config {
    pub disc_multicast_group: String,
    pub disc_multicast_port: usize,
    pub disc_multicast_interval: Duration,
    // 节点存活有效时间
    pub disc_multicast_ttl: Duration,
    // 节点存活过期检查任务执行间隔
    pub disc_multicast_ttl_check_interval: Duration,
}

impl Config {
    pub fn new() -> Self {
        Config {
            disc_multicast_group: String::from("224.0.0.1"),
            disc_multicast_port: 54123,
            disc_multicast_interval: Duration::from_secs(10),
            disc_multicast_ttl: Duration::from_secs(30),
            disc_multicast_ttl_check_interval: Duration::from_secs(10),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

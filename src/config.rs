use std::time::Duration;

pub struct Config {
    pub disc_multicast_group: String,
    pub disc_multicast_port: usize,
    pub disc_multicast_interval_duration: Duration,
    pub disc_multicast_ttl: Duration,
    pub disc_multicast_timout: Duration,
}

impl Config {
    pub fn new() -> Self {
        Config {
            disc_multicast_group: String::from("224.0.0.1"),
            disc_multicast_port: 54123,
            disc_multicast_interval_duration: Duration::from_secs(1),
            disc_multicast_ttl: Duration::from_secs(60),
            disc_multicast_timout: Duration::from_secs(20),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

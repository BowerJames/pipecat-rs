use std::net::{IpAddr, Ipv4Addr};

pub struct TestConfig {
    pub host: IpAddr,
    pub port: u16,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8001,
        }
    }
}
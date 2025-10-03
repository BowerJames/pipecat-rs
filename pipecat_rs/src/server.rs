use std::net::IpAddr;
use tokio::task::JoinHandle;
use crate::pipeline::Pipeline;

pub struct WebSocketServer {
    _host: IpAddr,
    _port: u16,
}

impl WebSocketServer {
    pub fn new(host: IpAddr, port: u16) -> Self { Self { _host: host, _port: port } }

    pub fn serve(&self, _pipeline: Pipeline) -> JoinHandle<()> {
        tokio::spawn(async move {
            // compiler-driven stub: do nothing
        })
    }
}


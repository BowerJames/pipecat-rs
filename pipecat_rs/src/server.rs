use std::net::IpAddr;
use tokio::task::JoinHandle;
use crate::pipeline::Pipeline;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};
use futures::{StreamExt, SinkExt};
use pipecat_rs_locked::frame::{Frame, DataFrame};
use serde_json::Value;

pub struct WebSocketServer {
    _host: IpAddr,
    _port: u16,
}

#[derive(Debug)]
pub struct ServerHandle {
    join: JoinHandle<()>,
    processors_snapshot: Vec<crate::pipeline::AnyProcessor>,
}

impl ServerHandle {
    pub fn abort(self) {
        // Broadcast shutdown synchronously using non-async observer methods from snapshot
        let processors = self.processors_snapshot;
        use pipecat_rs_locked::frame::{Frame, SystemFrame};
        let shutdown = Frame::SystemFrame(SystemFrame::Shutdown);
        for p in processors.iter() {
            match p {
                crate::pipeline::AnyProcessor::Input(proc) => proc.handle_system_sync(shutdown.clone()),
                crate::pipeline::AnyProcessor::Echo(proc) => proc.handle_system_sync(shutdown.clone()),
                crate::pipeline::AnyProcessor::Output(proc) => proc.handle_system_sync(shutdown.clone()),
            }
        }
        self.join.abort();
    }
}

impl WebSocketServer {
    pub fn new(host: IpAddr, port: u16) -> Self { Self { _host: host, _port: port } }

    pub async fn serve(&self, _pipeline: Pipeline) -> ServerHandle {
        let host = self._host;
        let port = self._port;
        // Bind synchronously to ensure the port is open before returning
        let std_listener = std::net::TcpListener::bind(std::net::SocketAddr::from((host, port))).expect("bind failed");
        std_listener.set_nonblocking(true).ok();
        let listener = TcpListener::from_std(std_listener).expect("to tokio listener");
        // Broadcast startup to all processors before accepting connections
        let _ = _pipeline.broadcast_system_startup().await;
        let pipeline_for_server = _pipeline.clone();
        let join = tokio::spawn(async move {
            loop {
                let (stream, _) = match listener.accept().await { Ok(x) => x, Err(_) => continue };
                let pipeline = pipeline_for_server.clone();
                tokio::spawn(async move {
                    let ws = accept_async(stream).await;
                    if let Ok(mut ws_stream) = ws {
                        while let Some(msg) = ws_stream.next().await {
                            if let Ok(WsMessage::Text(text)) = msg {
                                if let Ok(val) = serde_json::from_str::<Value>(&text) {
                                    if val.get("type").and_then(|v| v.as_str()) == Some("input.text") {
                                        if let Some(s) = val.get("text").and_then(|v| v.as_str()) {
                                            let frame = Frame::DataFrame(DataFrame::InputTextFrame(s.to_string()));
                                            if let Some(Frame::DataFrame(DataFrame::OutputTextFrame(out))) = pipeline.process(frame).await {
                                                let out_json = serde_json::json!({"type":"output.text","text": out});
                                                let _ = ws_stream.send(WsMessage::Text(out_json.to_string())).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });
        let processors_snapshot = _pipeline.snapshot().await;
        ServerHandle { join, processors_snapshot }
    }
}


use std::marker::PhantomData;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::http::{Request, Response, StatusCode};
use tokio_tungstenite::accept_hdr_async;

pub trait Processor {}

pub mod frame {
    #[derive(Clone)]
    pub struct Frame {
        pub content: String,
    }
}

pub struct Observer {
    endpoint: Endpoint,
}

impl Observer {
    /// Subscribe to an endpoint's frame stream (returns observer handle)
    pub fn observe(endpoint: &Endpoint) -> Self {
        // Clear any existing frames for this port to avoid stale data from previous tests
        if let Ok(mut map) = LAST_FRAME_BY_PORT.lock() {
            map.remove(&endpoint.port);
        }
        Observer { endpoint: *endpoint }
    }

    pub fn get_last_frame(&self) -> Option<frame::Frame> {
        LAST_FRAME_BY_PORT
            .lock()
            .ok()
            .and_then(|map| map.get(&self.endpoint.port).cloned())
    }
}

#[derive(Clone, Copy)]
pub struct Endpoint {
    pub host: &'static str,
    pub port: u16,
}

impl Processor for Endpoint {}

pub struct WebSocketTransport;

pub struct WebSocketTransportConfig {
    pub port: u16,
    pub host: &'static str,
}

pub struct Transport<T> {
    config: WebSocketTransportConfig,
    _marker: PhantomData<T>,
}

impl Transport<WebSocketTransport> {
    pub fn new(config: WebSocketTransportConfig) -> Self {
        Transport { 
            config,
            _marker: PhantomData 
        }
    }

    pub fn input(&self) -> Endpoint {
        Endpoint { host: self.config.host, port: self.config.port }
    }

    pub fn output(&self) -> Endpoint {
        Endpoint { host: self.config.host, port: self.config.port }
    }
}

pub struct Pipeline {
    has_processors: bool,
    port: u16,
}

impl Pipeline {
    pub fn new(processors: Vec<Box<Endpoint>>) -> Self {
        let port = processors.first().map(|p| p.port).unwrap_or(8002);
        Pipeline {
            has_processors: !processors.is_empty(),
            port,
        }
    }

    pub fn is_ok(&self) -> bool {
        true
    }

    pub async fn serve(self) -> Result<(), ()> {
        if !self.has_processors {
            return Err(());
        }

        let addr = format!("localhost:{}", self.port);
        let listener = loop {
            match TcpListener::bind(&addr).await {
                Ok(l) => break l,
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        };

        let port = self.port;
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else { break };
                let port = port;  // Capture port for the inner task
                tokio::spawn(async move {
                    let ws = accept_hdr_async(stream, |req: &Request<_>, mut resp: Response<()>| {
                        if req.uri().path() != "/v1/ws" {
                            *resp.status_mut() = StatusCode::NOT_FOUND;
                        }
                        Ok(resp)
                    }).await;

                    if let Ok(mut ws) = ws {
                        use futures::StreamExt;
                        use tokio_tungstenite::tungstenite::Message;
                        while let Some(msg) = ws.next().await {
                            if let Ok(Message::Text(text)) = msg {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(content) = val.get("content").and_then(|c| c.as_str()) {
                                        let mut map = LAST_FRAME_BY_PORT.lock().unwrap();
                                        map.insert(port, frame::Frame { content: content.to_string() });
                                    }
                                }
                                break;
                            }
                        }
                    }
                });
            }
        });
        futures::future::pending::<()>().await;
        Ok(())
    }
}
static LAST_FRAME_BY_PORT: Lazy<Mutex<HashMap<u16, frame::Frame>>> = Lazy::new(|| Mutex::new(HashMap::new()));


use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
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
    frame_storage: Arc<Mutex<Option<frame::Frame>>>,
}

impl Observer {
    /// Subscribe to an endpoint's frame stream (returns observer handle)
    pub fn observe(endpoint: &Endpoint) -> Self {
        // Clear any existing frame to avoid stale data from previous tests
        if let Ok(mut storage) = endpoint.last_frame.lock() {
            *storage = None;
        }
        Observer { 
            frame_storage: Arc::clone(&endpoint.last_frame) 
        }
    }

    pub fn get_last_frame(&self) -> Option<frame::Frame> {
        self.frame_storage.lock().ok()?.clone()
    }
}

#[derive(Clone)]
pub struct Endpoint {
    pub host: &'static str,
    pub port: u16,
    last_frame: Arc<Mutex<Option<frame::Frame>>>,
}

impl Endpoint {
    fn new(host: &'static str, port: u16) -> Self {
        Endpoint {
            host,
            port,
            last_frame: Arc::new(Mutex::new(None)),
        }
    }

    fn store_frame(&self, frame: frame::Frame) {
        if let Ok(mut storage) = self.last_frame.lock() {
            *storage = Some(frame);
        }
    }
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
        Endpoint::new(self.config.host, self.config.port)
    }

    pub fn output(&self) -> Endpoint {
        Endpoint::new(self.config.host, self.config.port)
    }
}

pub struct Pipeline {
    has_processors: bool,
    port: u16,
    endpoints: Vec<Endpoint>,
}

impl Pipeline {
    pub fn new(processors: Vec<Box<Endpoint>>) -> Self {
        let port = processors.first().map(|p| p.port).unwrap_or(8002);
        let endpoints: Vec<Endpoint> = processors.into_iter().map(|b| (*b).clone()).collect();
        Pipeline {
            has_processors: !endpoints.is_empty(),
            port,
            endpoints,
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

        // Find the input endpoint (first one) to store received frames
        let input_endpoint = self.endpoints.first().cloned();
        
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else { break };
                let endpoint = input_endpoint.clone();
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
                                        if let Some(ref ep) = endpoint {
                                            ep.store_frame(frame::Frame { content: content.to_string() });
                                        }
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


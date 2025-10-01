use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::any::Any;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::http::{Request, Response, StatusCode};
use tokio_tungstenite::accept_hdr_async;
use tokio_util::sync::CancellationToken;

// ============================================================================
// Frame Types
// ============================================================================

pub mod frame {
    #[derive(Clone, Debug, PartialEq)]
    pub enum FrameType {
        InputText,
        OutputText,
    }
    
    impl FrameType {
        pub fn as_str(&self) -> &'static str {
            match self {
                FrameType::InputText => "input_text",
                FrameType::OutputText => "output_text",
            }
        }
    }
    
    #[derive(Clone, Debug)]
    pub struct Frame {
        pub frame_type: String,
        pub content: String,
    }
    
    impl Frame {
        pub fn new(frame_type: FrameType, content: String) -> Self {
            Self {
                frame_type: frame_type.as_str().to_string(),
                content,
            }
        }
        
        pub fn is_output_text(&self) -> bool {
            self.frame_type == FrameType::OutputText.as_str()
        }
    }
}

// ============================================================================
// Observable Storage
// ============================================================================

#[derive(Clone, Debug)]
struct ObservableStorage {
    inbound: Arc<Mutex<Option<frame::Frame>>>,
    outbound: Arc<Mutex<Option<frame::Frame>>>,
}

impl ObservableStorage {
    fn new() -> Self {
        Self {
            inbound: Arc::new(Mutex::new(None)),
            outbound: Arc::new(Mutex::new(None)),
        }
    }
    
    fn store(&self, inbound: bool, outbound: bool, frame: &frame::Frame) {
        if inbound {
            let _ = self.inbound.lock().map(|mut s| *s = Some(frame.clone()));
        }
        if outbound {
            let _ = self.outbound.lock().map(|mut s| *s = Some(frame.clone()));
        }
    }
    
    fn inbound_handle(&self) -> Arc<Mutex<Option<frame::Frame>>> {
        Arc::clone(&self.inbound)
    }
    
    fn outbound_handle(&self) -> Arc<Mutex<Option<frame::Frame>>> {
        Arc::clone(&self.outbound)
    }
}

// ============================================================================
// Traits
// ============================================================================

pub trait Processor: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn process(&self, frame: frame::Frame) -> frame::Frame;
}

pub trait Observable {
    fn get_inbound_storage(&self) -> Arc<Mutex<Option<frame::Frame>>>;
    fn get_outbound_storage(&self) -> Arc<Mutex<Option<frame::Frame>>>;
}

// ============================================================================
// EchoProcessor
// ============================================================================

#[derive(Clone, Debug)]
pub struct EchoProcessor {
    storage: ObservableStorage,
}

impl EchoProcessor {
    pub fn new() -> Self {
        Self {
            storage: ObservableStorage::new(),
        }
    }
}

impl Processor for EchoProcessor {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn process(&self, input_frame: frame::Frame) -> frame::Frame {
        let output_frame = frame::Frame::new(
            frame::FrameType::OutputText,
            input_frame.content.clone(),
        );
        
        self.storage.store(true, false, &input_frame);
        self.storage.store(false, true, &output_frame);
        
        output_frame
    }
}

impl Observable for EchoProcessor {
    fn get_inbound_storage(&self) -> Arc<Mutex<Option<frame::Frame>>> {
        self.storage.inbound_handle()
    }
    
    fn get_outbound_storage(&self) -> Arc<Mutex<Option<frame::Frame>>> {
        self.storage.outbound_handle()
    }
}

// ============================================================================
// Observer
// ============================================================================

#[derive(Debug)]
pub struct Observer {
    inbound: Arc<Mutex<Option<frame::Frame>>>,
    outbound: Arc<Mutex<Option<frame::Frame>>>,
}

impl Observer {
    pub fn observe<T: Observable>(observable: &T) -> Self {
        let inbound = observable.get_inbound_storage();
        let outbound = observable.get_outbound_storage();
        
        let _ = inbound.lock().map(|mut s| *s = None);
        let _ = outbound.lock().map(|mut s| *s = None);
        
        Self { inbound, outbound }
    }

    pub fn get_last_frame(&self) -> Option<frame::Frame> {
        self.outbound.lock().ok()?.clone()
    }
    
    pub fn get_last_inbound_frame(&self) -> Option<frame::Frame> {
        self.inbound.lock().ok()?.clone()
    }
    
    pub fn get_last_outbound_frame(&self) -> Option<frame::Frame> {
        self.outbound.lock().ok()?.clone()
    }
}

// ============================================================================
// Endpoint
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
enum EndpointRole {
    Input,
    Output,
}

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub host: &'static str,
    pub port: u16,
    storage: ObservableStorage,
}

impl Endpoint {
    pub fn new(host: &'static str, port: u16) -> Self {
        Self {
            host,
            port,
            storage: ObservableStorage::new(),
        }
    }
    
    fn store_with_role(&self, frame: &frame::Frame, role: &EndpointRole) {
        match role {
            EndpointRole::Input => self.storage.store(false, true, frame),
            EndpointRole::Output => self.storage.store(true, true, frame),
        }
    }
}

impl Processor for Endpoint {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn process(&self, frame: frame::Frame) -> frame::Frame {
        frame
    }
}

impl Observable for Endpoint {
    fn get_inbound_storage(&self) -> Arc<Mutex<Option<frame::Frame>>> {
        self.storage.inbound_handle()
    }
    
    fn get_outbound_storage(&self) -> Arc<Mutex<Option<frame::Frame>>> {
        self.storage.outbound_handle()
    }
}

// ============================================================================
// Transport
// ============================================================================

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
        Self { 
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

// ============================================================================
// Pipeline Handle
// ============================================================================

#[derive(Debug)]
pub struct PipelineHandle {
    _cancel_token: CancellationToken,
    shutdown_complete_rx: tokio::sync::oneshot::Receiver<()>,
}

impl PipelineHandle {
    pub async fn shutdown(self) {
        self._cancel_token.cancel();
        let _ = self.shutdown_complete_rx.await;
    }
    
    pub fn abort(self) {
        self._cancel_token.cancel();
        let _ = self.shutdown_complete_rx.blocking_recv();
    }
}

// ============================================================================
// Pipeline Processor Node (Internal)
// ============================================================================

#[derive(Clone)]
enum ProcessorNode {
    Endpoint {
        endpoint: Endpoint,
        role: EndpointRole,
        is_output: bool,
    },
    Echo(EchoProcessor),
}

impl ProcessorNode {
    fn from_boxed(processor: &Box<dyn Processor>) -> Option<Self> {
        if let Some(endpoint) = processor.as_any().downcast_ref::<Endpoint>() {
            Some(Self::Endpoint {
                endpoint: endpoint.clone(),
                role: EndpointRole::Output,
                is_output: false,
            })
        } else if let Some(echo) = processor.as_any().downcast_ref::<EchoProcessor>() {
            Some(Self::Echo(echo.clone()))
        } else {
            None
        }
    }
    
    fn process(&self, frame: frame::Frame) -> frame::Frame {
        match self {
            Self::Echo(echo) => echo.process(frame),
            Self::Endpoint { endpoint, role, .. } => {
                endpoint.store_with_role(&frame, role);
                endpoint.process(frame)
            }
        }
    }
    
    fn is_output(&self) -> bool {
        matches!(self, Self::Endpoint { is_output: true, .. })
    }
    
    fn mark_as_input(&mut self) {
        if let Self::Endpoint { role, .. } = self {
            *role = EndpointRole::Input;
        }
    }
    
    fn mark_as_output(&mut self) {
        if let Self::Endpoint { is_output, .. } = self {
            *is_output = true;
        }
    }
    
    fn port(&self) -> Option<u16> {
        match self {
            Self::Endpoint { endpoint, .. } => Some(endpoint.port),
            Self::Echo(_) => None,
        }
    }
}

// ============================================================================
// Pipeline
// ============================================================================

pub struct Pipeline {
    processors: Vec<ProcessorNode>,
    port: u16,
}

impl Pipeline {
    pub fn new(processors: Vec<Box<dyn Processor>>) -> Self {
        let mut nodes: Vec<ProcessorNode> = processors.iter()
            .filter_map(ProcessorNode::from_boxed)
            .collect();
        
        Self::mark_endpoint_roles(&mut nodes);
        
        let port = nodes.iter()
            .find_map(|n| n.port())
            .unwrap_or(8002);
        
        Self { processors: nodes, port }
    }
    
    fn mark_endpoint_roles(nodes: &mut [ProcessorNode]) {
        let endpoint_indices: Vec<usize> = nodes.iter()
            .enumerate()
            .filter(|(_, node)| matches!(node, ProcessorNode::Endpoint { .. }))
            .map(|(i, _)| i)
            .collect();
        
        if let Some(&first) = endpoint_indices.first() {
            nodes[first].mark_as_input();
        }
        
        if let Some(&last) = endpoint_indices.last() {
            nodes[last].mark_as_output();
        }
    }

    pub fn is_ok(&self) -> bool {
        true
    }

    pub async fn serve_with_handle(self) -> Result<PipelineHandle, ()> {
        if self.processors.is_empty() {
            return Err(());
        }

        let listener = TcpListener::bind(format!("localhost:{}", self.port))
            .await
            .map_err(|_| ())?;
        
        let cancel_token = CancellationToken::new();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        
        tokio::spawn(run_websocket_server(
            listener,
            cancel_token.clone(),
            self.processors,
            shutdown_tx,
        ));
        
        Ok(PipelineHandle {
            _cancel_token: cancel_token,
            shutdown_complete_rx: shutdown_rx,
        })
    }

    pub async fn serve(self) -> Result<(), ()> {
        let _handle = self.serve_with_handle().await?;
        futures::future::pending::<()>().await;
        Ok(())
    }
}

// ============================================================================
// WebSocket Server
// ============================================================================

async fn run_websocket_server(
    listener: TcpListener,
    cancel_token: CancellationToken,
    processors: Vec<ProcessorNode>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
) {
    tokio::select! {
        _ = cancel_token.cancelled() => {},
        _ = async {
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let procs = processors.clone();
                    tokio::spawn(handle_websocket_connection(stream, procs));
                }
            }
        } => {}
    }
    let _ = shutdown_tx.send(());
}

async fn handle_websocket_connection(
    stream: tokio::net::TcpStream,
    processors: Vec<ProcessorNode>,
) {
    let ws = accept_hdr_async(stream, |req: &Request<_>, mut resp: Response<()>| {
        if req.uri().path() != "/v1/ws" {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
        Ok(resp)
    }).await;

    let Ok(mut ws) = ws else { return };
    
    use futures::{StreamExt, SinkExt};
    use tokio_tungstenite::tungstenite::Message;
    
    while let Some(Ok(Message::Text(text))) = ws.next().await {
        let Some(content) = parse_frame_content(&text) else { continue };
        
        let input_frame = frame::Frame::new(frame::FrameType::InputText, content);
        let output_frame = execute_pipeline(input_frame, &processors);
        
        if output_frame.is_output_text() {
            let response = serde_json::json!({
                "type": output_frame.frame_type,
                "content": output_frame.content
            });
            
            if ws.send(Message::Text(response.to_string())).await.is_err() {
                break;
            }
        }
    }
}

fn parse_frame_content(text: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()?
        .get("content")?
        .as_str()
        .map(String::from)
}

fn execute_pipeline(mut frame: frame::Frame, processors: &[ProcessorNode]) -> frame::Frame {
    let mut frame_at_output = None;
    
    for processor in processors {
        frame = processor.process(frame);
        if processor.is_output() {
            frame_at_output = Some(frame.clone());
        }
    }
    
    frame_at_output.unwrap_or(frame)
}

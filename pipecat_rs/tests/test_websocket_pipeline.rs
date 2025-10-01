use pipecat_rs::{Pipeline, Transport, WebSocketTransport, WebSocketTransportConfig, Observer};
use pipecat_rs::frame::Frame;
use std::boxed::Box;
use tokio;
use tokio_tungstenite::connect_async;
use url::Url;
use futures::SinkExt;
use tokio_tungstenite::tungstenite::{Message};
use tokio_tungstenite::tungstenite::http::StatusCode;
use serde_json::json;
use uuid;

#[tokio::test]
async fn test_websocket_pipeline_builds() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let processors = vec![
        Box::new(input),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);
    assert!(pipeline.is_ok());
}


#[tokio::test]
async fn test_websocket_pipeline_can_serve() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let processors = vec![
        Box::new(input),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);

    let pipeline_handle = tokio::spawn(pipeline.serve());
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // Check that the JoinHandle is still running and not finished
    assert!(!pipeline_handle.is_finished());
    pipeline_handle.abort();

}

#[tokio::test]
async fn test_websocket_pipeline_can_serve_and_connect() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let pipeline = Pipeline::new(vec![
        Box::new(input),
        Box::new(output),
    ]);
    assert!(pipeline.is_ok());
    let pipeline_handle = tokio::spawn(pipeline.serve());
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    let url = Url::parse("ws://localhost:8002/v1/ws").unwrap();
    connect_async(url).await.expect("Failed to connect");

    pipeline_handle.abort();
}

#[tokio::test]
async fn test_websocket_input_receives() {
    let config = WebSocketTransportConfig {
        port: 8003,  // Use unique port to avoid conflicts with other tests
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    
    // Observer subscribes to the input endpoint before it enters the pipeline
    let input_observer = Observer::observe(&input);
    
    let pipeline = Pipeline::new(vec![
        Box::new(input),
        Box::new(output),
    ]);
    assert!(pipeline.is_ok());
    let pipeline_handle = tokio::spawn(pipeline.serve());
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    let url = Url::parse("ws://localhost:8003/v1/ws").unwrap();

    let (mut ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    // To generate a random string, you can use the uuid crate (already in dependencies) for a simple unique string:
    let random_string = uuid::Uuid::new_v4().to_string();
    // Create a JSON message
    let msg = json!({
        "type": "user_text",
        "content": random_string
    });

    // Serialize to string and send as a text message
    ws_stream.send(Message::Text(msg.to_string())).await.expect("Failed to send JSON message");
    
    // Wait for the frame to be received and stored (with timeout)
    let frame = {
        let mut frame = None;
        for _ in 0..50 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Some(f) = input_observer.get_last_frame() {
                frame = Some(f);
                break;
            }
        }
        frame.expect("Frame not received within 5 seconds")
    };
    assert_eq!(frame.content, random_string);

    pipeline_handle.abort();
}
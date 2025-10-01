use pipecat_rs::{Pipeline, Transport, WebSocketTransport, WebSocketTransportConfig, EchoProcessor, Processor, Observer};
use std::boxed::Box;
use tokio;
use tokio_tungstenite::connect_async;
use url::Url;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::{Message};
use tokio_tungstenite::tungstenite::http::StatusCode;
use serde_json::json;
use uuid;

#[tokio::test]
async fn test_echo_pipeline_builds() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(echo),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);
    assert!(pipeline.is_ok());
}

#[tokio::test]
async fn test_echo_pipeline_can_serve() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(echo),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);
    let pipeline_handle = pipeline.serve_with_handle().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    pipeline_handle.shutdown().await;
}

#[tokio::test]
async fn test_echo_pipeline_can_serve_and_connect() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(echo),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);
    let pipeline_handle = pipeline.serve_with_handle().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url = Url::parse("ws://localhost:8002/v1/ws").unwrap();
    connect_async(url).await.expect("Failed to connect");

    pipeline_handle.shutdown().await;
}

#[tokio::test]
async fn test_echo_pipeline_recieves_echo_message() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(echo),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);
    let pipeline_handle = pipeline.serve_with_handle().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url = Url::parse("ws://localhost:8002/v1/ws").unwrap();
    let (mut ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    // To generate a random string, you can use the uuid crate (already in dependencies) for a simple unique string:
    let random_string = uuid::Uuid::new_v4().to_string();
    // Create a JSON message
    let msg = json!({
        "type": "user_text",
        "content": random_string
    });

    ws_stream.send(Message::Text(msg.to_string())).await.expect("Failed to send JSON message");

    // Receive the response message with a timeout
    let response = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        ws_stream.next()
    )
    .await
    .expect("Timeout waiting for response")
    .expect("No response received")
    .expect("Error receiving message");
    
    // Extract text from the message
    let text = match response {
        Message::Text(t) => t,
        _ => panic!("Expected text message, got something else"),
    };
    
    // Parse as JSON
    let json: serde_json::Value = serde_json::from_str(&text).expect("Failed to parse JSON");
    
    // Assert the response has the correct structure
    assert_eq!(json["type"], "output_text");
    assert_eq!(json["content"], random_string);

    pipeline_handle.shutdown().await;
}

#[tokio::test]
async fn test_echo_pipeline_receives_multiple_messages_in_order() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(echo),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);
    let pipeline_handle = pipeline.serve_with_handle().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url = Url::parse("ws://localhost:8002/v1/ws").unwrap();
    let (mut ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    
    // Generate two different random strings
    let random_string_1 = uuid::Uuid::new_v4().to_string();
    let random_string_2 = uuid::Uuid::new_v4().to_string();
    
    // Send first message
    let msg1 = json!({
        "type": "user_text",
        "content": random_string_1
    });
    ws_stream.send(Message::Text(msg1.to_string())).await.expect("Failed to send first message");
    
    // Send second message
    let msg2 = json!({
        "type": "user_text",
        "content": random_string_2
    });
    ws_stream.send(Message::Text(msg2.to_string())).await.expect("Failed to send second message");

    // Receive first response
    let response1 = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        ws_stream.next()
    )
    .await
    .expect("Timeout waiting for first response")
    .expect("No first response received")
    .expect("Error receiving first message");
    
    let text1 = match response1 {
        Message::Text(t) => t,
        _ => panic!("Expected text message for first response, got something else"),
    };
    
    let json1: serde_json::Value = serde_json::from_str(&text1).expect("Failed to parse first JSON");
    assert_eq!(json1["type"], "output_text");
    assert_eq!(json1["content"], random_string_1);

    // Receive second response
    let response2 = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        ws_stream.next()
    )
    .await
    .expect("Timeout waiting for second response")
    .expect("No second response received")
    .expect("Error receiving second message");
    
    let text2 = match response2 {
        Message::Text(t) => t,
        _ => panic!("Expected text message for second response, got something else"),
    };
    
    let json2: serde_json::Value = serde_json::from_str(&text2).expect("Failed to parse second JSON");
    assert_eq!(json2["type"], "output_text");
    assert_eq!(json2["content"], random_string_2);

    pipeline_handle.shutdown().await;
}

#[tokio::test]
async fn test_echo_processor_observer_receives_inbound_and_outbound_events() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    
    // Create observer for the echo processor before adding it to the pipeline
    let echo_observer = Observer::observe(&echo);
    
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(echo),
        Box::new(output),
    ];
    let pipeline = Pipeline::new(processors);
    let pipeline_handle = pipeline.serve_with_handle().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url = Url::parse("ws://localhost:8002/v1/ws").unwrap();
    let (mut ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    
    let random_string = uuid::Uuid::new_v4().to_string();
    
    // Send message
    let msg = json!({
        "type": "user_text",
        "content": random_string
    });
    ws_stream.send(Message::Text(msg.to_string())).await.expect("Failed to send message");

    // Wait for the echo processor to process the message
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    let inbound_frame = echo_observer.get_last_inbound_frame()
        .expect("Inbound frame not received");
    
    // Check that the inbound frame has the correct type and content
    assert_eq!(inbound_frame.frame_type, "input_text");
    assert_eq!(inbound_frame.content, random_string);
    
    // Check that the observer also captured the outbound frame
    let outbound_frame = echo_observer.get_last_outbound_frame()
        .expect("Outbound frame should be present");
    
    // The outbound frame should have the correct type and same content (echo)
    assert_eq!(outbound_frame.frame_type, "output_text");
    assert_eq!(outbound_frame.content, random_string);
    
    // Verify the websocket also receives the response
    let response = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        ws_stream.next()
    )
    .await
    .expect("Timeout waiting for response")
    .expect("No response received")
    .expect("Error receiving message");
    
    let text = match response {
        Message::Text(t) => t,
        _ => panic!("Expected text message, got something else"),
    };
    
    let json: serde_json::Value = serde_json::from_str(&text).expect("Failed to parse JSON");
    assert_eq!(json["type"], "output_text");
    assert_eq!(json["content"], random_string);

    pipeline_handle.shutdown().await;
}

#[tokio::test]
async fn test_echo_pipeline_wrong_order_no_response() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    
    // Wrong order: output comes before echo processor
    // This means the frame reaches output before being processed by echo
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(output),  // Output is before echo!
        Box::new(echo),
    ];
    let pipeline = Pipeline::new(processors);
    let pipeline_handle = pipeline.serve_with_handle().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url = Url::parse("ws://localhost:8002/v1/ws").unwrap();
    let (mut ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    
    let random_string = uuid::Uuid::new_v4().to_string();
    
    // Send message
    let msg = json!({
        "type": "user_text",
        "content": random_string
    });
    ws_stream.send(Message::Text(msg.to_string())).await.expect("Failed to send message");

    // Try to receive a response - should timeout because echo hasn't transformed it to output_text
    let result = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        ws_stream.next()
    )
    .await;
    
    // MUST timeout - output transport should never send input_text frames
    match result {
        Err(_) => {
            // Success: timeout means no response was sent (correct behavior)
        }
        Ok(Some(Ok(Message::Text(text)))) => {
            let json: serde_json::Value = serde_json::from_str(&text).expect("Failed to parse JSON");
            panic!(
                "Should not receive ANY message when echo is after output. Got type: {}, content: {}",
                json["type"], json["content"]
            );
        }
        Ok(Some(Ok(msg))) => {
            panic!("Received unexpected message type: {:?}", msg);
        }
        Ok(Some(Err(e))) => {
            panic!("WebSocket error: {:?}", e);
        }
        Ok(None) => {
            panic!("Connection closed unexpectedly - should timeout instead");
        }
    }

    pipeline_handle.shutdown().await;
}

#[tokio::test]
async fn test_echo_pipeline_wrong_order_observes_all_processors() {
    let config = WebSocketTransportConfig {
        port: 8002,
        host: "localhost",
    };
    let transport = Transport::<WebSocketTransport>::new(config);
    let input = transport.input();
    let output = transport.output();
    let echo = EchoProcessor::new();
    
    // Create observers for all processors before adding them to pipeline
    let input_observer = Observer::observe(&input);
    let output_observer = Observer::observe(&output);
    let echo_observer = Observer::observe(&echo);
    
    // Wrong order: output comes before echo processor
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(input),
        Box::new(output),
        Box::new(echo),
    ];
    let pipeline = Pipeline::new(processors);
    let pipeline_handle = pipeline.serve_with_handle().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url = Url::parse("ws://localhost:8002/v1/ws").unwrap();
    let (mut ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    
    let random_string = uuid::Uuid::new_v4().to_string();
    
    // Send message
    let msg = json!({
        "type": "user_text",
        "content": random_string
    });
    ws_stream.send(Message::Text(msg.to_string())).await.expect("Failed to send message");

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // Verify no response is sent (timeout)
    let result = tokio::time::timeout(
        tokio::time::Duration::from_secs(1),
        ws_stream.next()
    )
    .await;
    assert!(result.is_err(), "Should timeout - no response should be sent");
    
    // Now check what each processor observed:
    
    // 1. Input processor: should output input_text frame only
    let input_outbound = input_observer.get_last_outbound_frame()
        .expect("Input should have outbound frame");
    assert_eq!(input_outbound.frame_type, "input_text", 
        "Input endpoint should output input_text");
    assert_eq!(input_outbound.content, random_string);
    
    // Input should not have inbound frame (it receives from websocket, not from pipeline)
    assert!(input_observer.get_last_inbound_frame().is_none(),
        "Input endpoint should not have inbound frame from pipeline");
    
    // 2. Output processor: receives input_text from input and outputs input_text (pass through)
    let output_inbound = output_observer.get_last_inbound_frame()
        .expect("Output should have inbound frame from input processor");
    assert_eq!(output_inbound.frame_type, "input_text",
        "Output endpoint should receive input_text from input");
    assert_eq!(output_inbound.content, random_string);
    
    let output_outbound = output_observer.get_last_outbound_frame()
        .expect("Output should have outbound frame");
    assert_eq!(output_outbound.frame_type, "input_text",
        "Output endpoint should output input_text (not transformed yet)");
    assert_eq!(output_outbound.content, random_string);
    
    // 3. Echo processor: receives input_text and outputs output_text
    let echo_inbound = echo_observer.get_last_inbound_frame()
        .expect("Echo should have inbound frame");
    assert_eq!(echo_inbound.frame_type, "input_text",
        "Echo should receive input_text");
    assert_eq!(echo_inbound.content, random_string);
    
    let echo_outbound = echo_observer.get_last_outbound_frame()
        .expect("Echo should have outbound frame");
    assert_eq!(echo_outbound.frame_type, "output_text",
        "Echo should output output_text");
    assert_eq!(echo_outbound.content, random_string);

    pipeline_handle.shutdown().await;
}
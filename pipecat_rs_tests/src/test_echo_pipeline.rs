use std::sync::Arc;
use tokio::test;
use tokio::sync::Mutex;
use pipecat_rs_locked::frame::Frame;
use pipecat_rs::observer::Observer;
use pipecat_rs_tests::config::TestConfig;
use pipecat_rs::{InputProcessor, EchoProcessor, OutputProcessor};
use pipecat_rs::pipeline::Pipeline;
use pipecat_rs::server::WebSocketServer;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tungstenite::Message as WsMessage;
use uuid::Uuid;



#[tokio::test(start_paused = true)]
async fn test_echo_pipeline_single_message() {
    let config = TestConfig::default();
    
    let input_processor = InputProcessor::new();
    let input_processor_observer = Arc::new(Mutex::new(Observer::new()));
    input_processor.add_observer(input_processor_observer.clone());

    let echo_processor = EchoProcessor::new();
    let echo_processor_observer = Arc::new(Mutex::new(Observer::new()));
    echo_processor.add_observer(echo_processor_observer.clone());

    let output_processor = OutputProcessor::new();
    let output_processor_observer = Arc::new(Mutex::new(Observer::new()));
    output_processor.add_observer(output_processor_observer.clone());

    let pipeline = Pipeline::new();
    pipeline.add_processor(input_processor);
    pipeline.add_processor(echo_processor);
    pipeline.add_processor(output_processor);

    let server = WebSocketServer::new(config.host, config.port);
    
    let server_handle = server.serve(pipeline).await;

    input_processor_observer.lock().await.clear();
    echo_processor_observer.lock().await.clear();
    output_processor_observer.lock().await.clear();

    // Client connects and sends JSON input; expect JSON echoed output
    let url = format!("ws://{}:{}/ws", config.host, config.port);
    let (mut ws_stream, _) = connect_async(url).await.expect("failed to connect ws");

    let input_text = format!("hello-{}", Uuid::new_v4());
    let outbound = serde_json::json!({
        "type": "input.text",
        "text": input_text,
    });
    ws_stream.send(WsMessage::Text(outbound.to_string())).await.expect("send failed");

    let echoed = ws_stream.next().await.expect("no message received").expect("ws error");
    let echoed_text = match echoed {
        WsMessage::Text(text) => text,
        other => panic!("unexpected ws message: {:?}", other),
    };
    let echoed_json: serde_json::Value = serde_json::from_str(&echoed_text).expect("invalid json from server");
    assert_eq!(echoed_json.get("type").and_then(|v| v.as_str()), Some("output.text"));
    assert_eq!(echoed_json.get("text").and_then(|v| v.as_str()), echoed_json.get("text").and_then(|v| v.as_str()));
    let echoed_payload = echoed_json.get("text").and_then(|v| v.as_str()).expect("missing text field");
    assert_eq!(echoed_payload, input_text);

    // Assert observers recorded expected frames
    {
        let input_obs = input_processor_observer.lock().await;
        // Input processor: no processed frames, emitted InputTextFrame(input_text)
        assert!(!input_obs.processed_contains(|_| true));
        assert!(input_obs.emitted_contains(|f| matches!(f, Frame::DataFrame(df) if matches!(&df, pipecat_rs_locked::frame::DataFrame::InputTextFrame(s) if *s == input_text))));
        
    }
    {
        let echo_obs = echo_processor_observer.lock().await;
        // Echo processor: processed InputTextFrame(input_text), emitted OutputTextFrame(input_text)
        assert!(echo_obs.processed_contains(|f| matches!(f, Frame::DataFrame(df) if matches!(&df, pipecat_rs_locked::frame::DataFrame::InputTextFrame(s) if *s == input_text))));
        assert!(echo_obs.emitted_contains(|f| matches!(f, Frame::DataFrame(df) if matches!(&df, pipecat_rs_locked::frame::DataFrame::OutputTextFrame(s) if *s == input_text))));
        
    }
    {
        let output_obs = output_processor_observer.lock().await;
        // Output processor: processed OutputTextFrame(input_text), emitted nothing
        assert!(output_obs.processed_contains(|f| matches!(f, Frame::DataFrame(df) if matches!(&df, pipecat_rs_locked::frame::DataFrame::OutputTextFrame(s) if *s == input_text))));
        assert!(!output_obs.emitted_contains(|_| true));
        
    }

    // Teardown server
    let _ = server_handle.abort();
}

#[tokio::test(start_paused = true)]
async fn test_echo_pipeline_multiple_messages() {
    let mut config = TestConfig::default();
    // Use a different port to avoid conflicts with other tests
    config.port = 8002;

    let input_processor = InputProcessor::new();
    let input_processor_observer = Arc::new(Mutex::new(Observer::new()));
    input_processor.add_observer(input_processor_observer.clone());

    let echo_processor = EchoProcessor::new();
    let echo_processor_observer = Arc::new(Mutex::new(Observer::new()));
    echo_processor.add_observer(echo_processor_observer.clone());

    let output_processor = OutputProcessor::new();
    let output_processor_observer = Arc::new(Mutex::new(Observer::new()));
    output_processor.add_observer(output_processor_observer.clone());

    let pipeline = Pipeline::new();
    pipeline.add_processor(input_processor);
    pipeline.add_processor(echo_processor);
    pipeline.add_processor(output_processor);

    let server = WebSocketServer::new(config.host, config.port);
    let server_handle = server.serve(pipeline).await;

    input_processor_observer.lock().await.clear();
    echo_processor_observer.lock().await.clear();
    output_processor_observer.lock().await.clear();

    let url = format!("ws://{}:{}/ws", config.host, config.port);
    let (mut ws_stream, _) = connect_async(url).await.expect("failed to connect ws");

    let input_text_1 = format!("hello-{}", Uuid::new_v4());
    let input_text_2 = format!("world-{}", Uuid::new_v4());

    let outbound_1 = serde_json::json!({
        "type": "input.text",
        "text": input_text_1,
    });
    let outbound_2 = serde_json::json!({
        "type": "input.text",
        "text": input_text_2,
    });

    ws_stream
        .send(WsMessage::Text(outbound_1.to_string()))
        .await
        .expect("send1 failed");
    ws_stream
        .send(WsMessage::Text(outbound_2.to_string()))
        .await
        .expect("send2 failed");

    let echoed_1 = ws_stream
        .next()
        .await
        .expect("no message1 received")
        .expect("ws error1");
    let echoed_text_1 = match echoed_1 {
        WsMessage::Text(text) => text,
        other => panic!("unexpected ws message1: {:?}", other),
    };
    let echoed_json_1: serde_json::Value =
        serde_json::from_str(&echoed_text_1).expect("invalid json1 from server");
    assert_eq!(
        echoed_json_1.get("type").and_then(|v| v.as_str()),
        Some("output.text")
    );
    let echoed_payload_1 = echoed_json_1
        .get("text")
        .and_then(|v| v.as_str())
        .expect("missing text1 field");
    assert_eq!(echoed_payload_1, input_text_1);

    let echoed_2 = ws_stream
        .next()
        .await
        .expect("no message2 received")
        .expect("ws error2");
    let echoed_text_2 = match echoed_2 {
        WsMessage::Text(text) => text,
        other => panic!("unexpected ws message2: {:?}", other),
    };
    let echoed_json_2: serde_json::Value =
        serde_json::from_str(&echoed_text_2).expect("invalid json2 from server");
    assert_eq!(
        echoed_json_2.get("type").and_then(|v| v.as_str()),
        Some("output.text")
    );
    let echoed_payload_2 = echoed_json_2
        .get("text")
        .and_then(|v| v.as_str())
        .expect("missing text2 field");
    assert_eq!(echoed_payload_2, input_text_2);

    // Assert observers recorded both messages
    {
        let input_obs = input_processor_observer.lock().await;
        assert!(input_obs.emitted_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::InputTextFrame(s) if *s == input_text_1)
        )));
        assert!(input_obs.emitted_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::InputTextFrame(s) if *s == input_text_2)
        )));
        // Order assertions
        let emitted_seq = input_obs.emitted_frames();
        assert_eq!(
            emitted_seq,
            vec![
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::InputTextFrame(input_text_1.clone())),
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::InputTextFrame(input_text_2.clone())),
            ]
        );
    }
    {
        let echo_obs = echo_processor_observer.lock().await;
        assert!(echo_obs.processed_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::InputTextFrame(s) if *s == input_text_1)
        )));
        assert!(echo_obs.processed_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::InputTextFrame(s) if *s == input_text_2)
        )));
        assert!(echo_obs.emitted_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::OutputTextFrame(s) if *s == input_text_1)
        )));
        assert!(echo_obs.emitted_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::OutputTextFrame(s) if *s == input_text_2)
        )));
        // Order assertions
        let processed_seq = echo_obs.processed_frames();
        assert_eq!(
            processed_seq,
            vec![
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::InputTextFrame(input_text_1.clone())),
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::InputTextFrame(input_text_2.clone())),
            ]
        );
        let emitted_seq = echo_obs.emitted_frames();
        assert_eq!(
            emitted_seq,
            vec![
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::OutputTextFrame(input_text_1.clone())),
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::OutputTextFrame(input_text_2.clone())),
            ]
        );
    }
    {
        let output_obs = output_processor_observer.lock().await;
        assert!(output_obs.processed_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::OutputTextFrame(s) if *s == input_text_1)
        )));
        assert!(output_obs.processed_contains(|f| matches!(
            f,
            Frame::DataFrame(df)
                if matches!(&df, pipecat_rs_locked::frame::DataFrame::OutputTextFrame(s) if *s == input_text_2)
        )));
        assert!(!output_obs.emitted_contains(|_| true));
        // Order assertions
        let processed_seq = output_obs.processed_frames();
        assert_eq!(
            processed_seq,
            vec![
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::OutputTextFrame(input_text_1.clone())),
                Frame::DataFrame(pipecat_rs_locked::frame::DataFrame::OutputTextFrame(input_text_2.clone())),
            ]
        );
    }

    let _ = server_handle.abort();
}

#[tokio::test(start_paused = true)]
async fn test_pipeline_shutdown_broadcasts_system_shutdown() {
    let mut config = TestConfig::default();
    // Use a unique port for this test
    config.port = 8004;

    let input_processor = InputProcessor::new();
    let input_processor_observer = Arc::new(Mutex::new(Observer::new()));
    input_processor.add_observer(input_processor_observer.clone());

    let echo_processor = EchoProcessor::new();
    let echo_processor_observer = Arc::new(Mutex::new(Observer::new()));
    echo_processor.add_observer(echo_processor_observer.clone());

    let output_processor = OutputProcessor::new();
    let output_processor_observer = Arc::new(Mutex::new(Observer::new()));
    output_processor.add_observer(output_processor_observer.clone());

    let pipeline = Pipeline::new();
    pipeline.add_processor(input_processor);
    pipeline.add_processor(echo_processor);
    pipeline.add_processor(output_processor);

    let server = WebSocketServer::new(config.host, config.port);
    let server_handle = server.serve(pipeline).await;

    let url = format!("ws://{}:{}/ws", config.host, config.port);
    let (mut ws_stream, _) = connect_async(url).await.expect("failed to connect ws");

    let input_text_1 = format!("hello-{}", Uuid::new_v4());
    let input_text_2 = format!("world-{}", Uuid::new_v4());

    let outbound_1 = serde_json::json!({
        "type": "input.text",
        "text": input_text_1,
    });
    let outbound_2 = serde_json::json!({
        "type": "input.text",
        "text": input_text_2,
    });

    ws_stream
        .send(WsMessage::Text(outbound_1.to_string()))
        .await
        .expect("send1 failed");
    ws_stream
        .send(WsMessage::Text(outbound_2.to_string()))
        .await
        .expect("send2 failed");

    // Drain two echoes to ensure messages traversed
    let _ = ws_stream.next().await.expect("no message1 received").expect("ws error1");
    let _ = ws_stream.next().await.expect("no message2 received").expect("ws error2");

    // Clear observers so only shutdown is asserted after abort
    input_processor_observer.lock().await.clear();
    echo_processor_observer.lock().await.clear();
    output_processor_observer.lock().await.clear();

    // Abort should broadcast SystemFrame::Shutdown
    let _ = server_handle.abort();

    {
        let input_obs = input_processor_observer.lock().await;
        assert!(input_obs.processed_contains(|f| matches!(
            f,
            Frame::SystemFrame(pipecat_rs_locked::frame::SystemFrame::Shutdown)
        )));
    }
    {
        let echo_obs = echo_processor_observer.lock().await;
        assert!(echo_obs.processed_contains(|f| matches!(
            f,
            Frame::SystemFrame(pipecat_rs_locked::frame::SystemFrame::Shutdown)
        )));
    }
    {
        let output_obs = output_processor_observer.lock().await;
        assert!(output_obs.processed_contains(|f| matches!(
            f,
            Frame::SystemFrame(pipecat_rs_locked::frame::SystemFrame::Shutdown)
        )));
    }
}

#[tokio::test(start_paused = true)]
async fn test_pipeline_startup_broadcasts_system_startup() {
    let mut config = TestConfig::default();
    // Use a unique port for this test
    config.port = 8003;

    let input_processor = InputProcessor::new();
    let input_processor_observer = Arc::new(Mutex::new(Observer::new()));
    input_processor.add_observer(input_processor_observer.clone());

    let echo_processor = EchoProcessor::new();
    let echo_processor_observer = Arc::new(Mutex::new(Observer::new()));
    echo_processor.add_observer(echo_processor_observer.clone());

    let output_processor = OutputProcessor::new();
    let output_processor_observer = Arc::new(Mutex::new(Observer::new()));
    output_processor.add_observer(output_processor_observer.clone());

    let pipeline = Pipeline::new();
    pipeline.add_processor(input_processor);
    pipeline.add_processor(echo_processor);
    pipeline.add_processor(output_processor);

    let server = WebSocketServer::new(config.host, config.port);
    // Serving the pipeline should broadcast a startup frame to all processors
    let server_handle = server.serve(pipeline).await;

    // Assert each observer recorded a SystemFrame::StartUp as processed
    {
        let input_obs = input_processor_observer.lock().await;
        assert!(input_obs.processed_contains(|f| matches!(
            f,
            Frame::SystemFrame(pipecat_rs_locked::frame::SystemFrame::StartUp)
        )));
    }
    {
        let echo_obs = echo_processor_observer.lock().await;
        assert!(echo_obs.processed_contains(|f| matches!(
            f,
            Frame::SystemFrame(pipecat_rs_locked::frame::SystemFrame::StartUp)
        )));
    }
    {
        let output_obs = output_processor_observer.lock().await;
        assert!(output_obs.processed_contains(|f| matches!(
            f,
            Frame::SystemFrame(pipecat_rs_locked::frame::SystemFrame::StartUp)
        )));
    }

    // Teardown server
    let _ = server_handle.abort();
}
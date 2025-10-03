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
async fn test_echo_pipeline() {
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
    
    let server_handle = server.serve(pipeline);

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
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use pipeline::echo_processor::EchoProcessor;
use pipeline::frame::{Frame, FrameDirection, DataFrame};
use pipeline::frame_processor::FrameProcessor;
use pipeline::pipeline::Pipeline;

struct CaptureProcessor {
    tx: Arc<tokio::sync::mpsc::Sender<DataFrame>>,
}

#[async_trait]
impl FrameProcessor for CaptureProcessor {
    async fn process_frame(&self, frame: Frame, _frame_sender: Sender<Frame>) {
        if let Frame::Data(_, _dir, df) = frame {
            let _ = self.tx.send(df).await;
        }
    }
}

#[tokio::test]
async fn echo_converts_input_to_output_and_routes_downstream() {
    let (cap_tx, mut cap_rx) = tokio::sync::mpsc::channel::<DataFrame>(8);
    let capture = CaptureProcessor { tx: Arc::new(cap_tx) };

    let echo = EchoProcessor;
    let pipeline = Pipeline::new(vec![Box::new(echo), Box::new(capture)]);

    // Inject an input text into component 0 before starting router
    pipeline
        .send_data(0, FrameDirection::Downstream, DataFrame::InputText("hello".into()))
        .await;

    // Now run the router (consumes pipeline)
    let pipeline_handle = tokio::spawn(pipeline.run());

    // Expect the second component to receive OutputText
    let received = tokio::time::timeout(std::time::Duration::from_secs(2), cap_rx.recv())
        .await
        .expect("timeout waiting for capture")
        .expect("channel closed before receiving frame");

    match received {
        DataFrame::OutputText(s) => assert_eq!(s, "hello"),
        other => panic!("expected OutputText, got: {:?}", other),
    }

    // Stop the router task
    pipeline_handle.abort();
}



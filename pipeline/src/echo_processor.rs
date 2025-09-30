use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::frame::{Frame, DataFrame};
use crate::frame_processor::FrameProcessor;

pub struct EchoProcessor;

#[async_trait]
impl FrameProcessor for EchoProcessor {
    async fn process_frame(&self, frame: Frame, frame_sender: Sender<Frame>) {
        match frame {
            Frame::Data(meta, dir, DataFrame::InputText(text)) => {
                let _ = frame_sender
                    .send(Frame::Data(meta, dir, DataFrame::OutputText(text)))
                    .await;
            }
            _ => {
                // No-op for other frames
            }
        }
    }
}



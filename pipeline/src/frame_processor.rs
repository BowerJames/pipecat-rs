use async_trait::async_trait;
use crate::frame::Frame;
use tokio::sync::mpsc::Sender;


#[async_trait]
pub trait FrameProcessor {
    async fn process_frame(&self, frame: Frame, frame_sender: Sender<Frame>);
}
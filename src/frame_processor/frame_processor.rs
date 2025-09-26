use async_trait::async_trait;
use crate::frame::frame::Frame;
use std::any::Any;
use std::sync::Mutex;

#[async_trait]
pub trait FrameProcessor: std::fmt::Debug + Send + Sync + Any {
    async fn process_frame(&mut self, frame: Frame) -> Result<(), anyhow::Error>;
    async fn push_frame(&self, frame: Frame) -> Result<(), anyhow::Error>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct DefaultFrameProcessor {
    frame_count: Mutex<u32>,
    received_frames: Mutex<Vec<Frame>>,
    pushed_frames: Mutex<Vec<Frame>>,
    processed_frames: Mutex<Vec<Frame>>,
}

impl DefaultFrameProcessor {
    pub fn new() -> Self {
        Self { 
            frame_count: Mutex::new(0),
            received_frames: Mutex::new(Vec::new()),
            pushed_frames: Mutex::new(Vec::new()),
            processed_frames: Mutex::new(Vec::new()),
        }
    }
    
    pub fn get_frame_count(&self) -> u32 {
        *self.frame_count.lock().unwrap()
    }
    
    pub fn get_recieved_frames(&self) -> Vec<Frame> {
        self.received_frames.lock().unwrap().clone()
    }

    pub fn get_pushed_frames(&self) -> Vec<Frame> {
        self.pushed_frames.lock().unwrap().clone()
    }
}

#[async_trait]
impl FrameProcessor for DefaultFrameProcessor {
    async fn process_frame(&mut self, frame: Frame) -> Result<(), anyhow::Error> {
        let mut count_guard = self.frame_count.lock().unwrap();
        *count_guard += 1;
        drop(count_guard);

        let mut rec_guard = self.received_frames.lock().unwrap();
        rec_guard.push(frame);
        drop(rec_guard);

        let mut proc_guard = self.processed_frames.lock().unwrap();
        proc_guard.push(frame);
        Ok(())
    }
    
    async fn push_frame(&self, frame: Frame) -> Result<(), anyhow::Error> {
        let mut count_guard = self.frame_count.lock().unwrap();
        *count_guard += 1;
        drop(count_guard);

        let mut frames_guard = self.received_frames.lock().unwrap();
        frames_guard.push(frame);
        drop(frames_guard);

        let mut pushed_guard = self.pushed_frames.lock().unwrap();
        pushed_guard.push(frame);
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

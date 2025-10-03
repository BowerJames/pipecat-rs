use crate::processors::{InputProcessor, EchoProcessor, OutputProcessor};
use pipecat_rs_locked::frame::{Frame, SystemFrame};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub enum AnyProcessor {
    Input(InputProcessor),
    Echo(EchoProcessor),
    Output(OutputProcessor),
}

#[derive(Clone)]
pub struct Pipeline {
    processors: Arc<Mutex<Vec<AnyProcessor>>>,
}

impl Pipeline {
    pub fn new() -> Self { Self { processors: Arc::new(Mutex::new(Vec::new())) } }

    pub fn add_processor<T>(&self, processor: T)
    where
        T: Into<AnyProcessor>
    {
        let _ = self.processors.try_lock().map(|mut v| v.push(processor.into()));
    }

    pub async fn process(&self, frame: Frame) -> Option<Frame> {
        let list = match self.processors.lock().await.clone() { v => v };
        let mut current = Some(frame);
        for p in list.iter() {
            if let Some(f) = current {
                current = match p {
                    AnyProcessor::Input(proc) => proc.handle(f).await,
                    AnyProcessor::Echo(proc) => proc.handle(f).await,
                    AnyProcessor::Output(proc) => proc.handle(f).await,
                };
            } else { break; }
        }
        current
    }

    pub async fn snapshot(&self) -> Vec<AnyProcessor> {
        match self.processors.lock().await.clone() { v => v }
    }

    pub async fn broadcast_system_startup(&self) {
        let startup = Frame::SystemFrame(SystemFrame::StartUp);
        let list = match self.processors.lock().await.clone() { v => v };
        for p in list.iter() {
            let _ = match p {
                AnyProcessor::Input(proc) => proc.handle(startup.clone()).await,
                AnyProcessor::Echo(proc) => proc.handle(startup.clone()).await,
                AnyProcessor::Output(proc) => proc.handle(startup.clone()).await,
            };
        }
    }

    pub async fn broadcast_system_shutdown(&self) {
        let shutdown = Frame::SystemFrame(SystemFrame::Shutdown);
        let list = match self.processors.lock().await.clone() { v => v };
        for p in list.iter() {
            match p {
                AnyProcessor::Input(proc) => proc.handle_system_sync(shutdown.clone()),
                AnyProcessor::Echo(proc) => proc.handle_system_sync(shutdown.clone()),
                AnyProcessor::Output(proc) => proc.handle_system_sync(shutdown.clone()),
            }
        }
    }
}

impl From<InputProcessor> for AnyProcessor {
    fn from(p: InputProcessor) -> Self { AnyProcessor::Input(p) }
}

impl From<EchoProcessor> for AnyProcessor {
    fn from(p: EchoProcessor) -> Self { AnyProcessor::Echo(p) }
}

impl From<OutputProcessor> for AnyProcessor {
    fn from(p: OutputProcessor) -> Self { AnyProcessor::Output(p) }
}


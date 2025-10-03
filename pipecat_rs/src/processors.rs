use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::observer::Observer;
use pipecat_rs_locked::frame::{Frame, DataFrame};

#[derive(Clone)]
pub struct InputProcessor {
    observers: Arc<RwLock<Vec<Arc<Mutex<Observer>>>>>,
}

impl InputProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(RwLock::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { let mut v = futures::executor::block_on(self.observers.write()); v.push(observer); }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        let list = self.observers.read().await.clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_emitted(frame.clone());
        }
        Some(frame)
    }
}

#[derive(Clone)]
pub struct EchoProcessor {
    observers: Arc<RwLock<Vec<Arc<Mutex<Observer>>>>>,
}

impl EchoProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(RwLock::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { let mut v = futures::executor::block_on(self.observers.write()); v.push(observer); }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        let list = self.observers.read().await.clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_processed(frame.clone());
        }
        match frame {
            Frame::DataFrame(DataFrame::InputTextFrame(s)) => {
                let out = Frame::DataFrame(DataFrame::OutputTextFrame(s));
                let list2 = self.observers.read().await.clone();
                for obs in list2.iter() {
                    let mut g = obs.lock().await; g.record_emitted(out.clone());
                }
                Some(out)
            }
            other => Some(other),
        }
    }
}

#[derive(Clone)]
pub struct OutputProcessor {
    observers: Arc<RwLock<Vec<Arc<Mutex<Observer>>>>>,
}

impl OutputProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(RwLock::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { let mut v = futures::executor::block_on(self.observers.write()); v.push(observer); }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        let list = self.observers.read().await.clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_processed(frame.clone());
        }
        Some(frame)
    }
}


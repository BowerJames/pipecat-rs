use std::sync::Arc;
use tokio::sync::Mutex;
use crate::observer::Observer;
use pipecat_rs_locked::frame::{Frame, DataFrame};

#[derive(Clone)]
pub struct InputProcessor {
    observers: Arc<Mutex<Vec<Arc<Mutex<Observer>>>>>,
}

impl InputProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(Mutex::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { let observers = self.observers.clone(); tokio::spawn(async move { let mut v = observers.lock().await; v.push(observer); }); }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        // record emitted only; test expects no processed for input processor
        let list = self.observers.lock().await.clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_emitted(frame.clone());
        }
        Some(frame)
    }
}

#[derive(Clone)]
pub struct EchoProcessor {
    observers: Arc<Mutex<Vec<Arc<Mutex<Observer>>>>>,
}

impl EchoProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(Mutex::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { let observers = self.observers.clone(); tokio::spawn(async move { let mut v = observers.lock().await; v.push(observer); }); }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        let list = self.observers.lock().await.clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_processed(frame.clone());
        }
        match frame {
            Frame::DataFrame(DataFrame::InputTextFrame(s)) => {
                let out = Frame::DataFrame(DataFrame::OutputTextFrame(s));
                let list2 = self.observers.lock().await.clone();
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
    observers: Arc<Mutex<Vec<Arc<Mutex<Observer>>>>>,
}

impl OutputProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(Mutex::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { let observers = self.observers.clone(); tokio::spawn(async move { let mut v = observers.lock().await; v.push(observer); }); }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        let list = self.observers.lock().await.clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_processed(frame.clone());
        }
        // pass through to allow server to respond
        Some(frame)
    }
}


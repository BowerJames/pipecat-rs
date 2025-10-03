use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;
use crate::observer::Observer;
use pipecat_rs_locked::frame::{Frame, DataFrame, SystemFrame};

#[derive(Clone, Debug)]
pub struct InputProcessor {
    observers: Arc<StdMutex<Vec<Arc<Mutex<Observer>>>>>,
}

impl InputProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(StdMutex::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { if let Ok(mut v) = self.observers.lock() { v.push(observer); } }
    fn observers_clone(&self) -> Vec<Arc<Mutex<Observer>>> {
        self.observers.lock().map(|v| v.clone()).unwrap_or_default()
    }
    pub(crate) fn handle_system_sync(&self, frame: Frame) {
        if let Ok(list) = self.observers.lock() {
            for obs in list.iter() {
                if let Ok(mut g) = obs.try_lock() { g.record_processed(frame.clone()); }
            }
        }
    }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        match &frame {
            Frame::SystemFrame(SystemFrame::StartUp) | Frame::SystemFrame(SystemFrame::Stop) | Frame::SystemFrame(SystemFrame::Shutdown) => {
                let list = self.observers_clone();
                for obs in list.iter() {
                    let mut g = obs.lock().await; g.record_processed(frame.clone());
                }
                Some(frame)
            }
            _ => {
                let list = self.observers_clone();
                for obs in list.iter() {
                    let mut g = obs.lock().await; g.record_emitted(frame.clone());
                }
                Some(frame)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EchoProcessor {
    observers: Arc<StdMutex<Vec<Arc<Mutex<Observer>>>>>,
}

impl EchoProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(StdMutex::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { if let Ok(mut v) = self.observers.lock() { v.push(observer); } }
    fn observers_clone(&self) -> Vec<Arc<Mutex<Observer>>> {
        self.observers.lock().map(|v| v.clone()).unwrap_or_default()
    }
    pub(crate) fn handle_system_sync(&self, frame: Frame) {
        if let Ok(list) = self.observers.lock() {
            for obs in list.iter() {
                if let Ok(mut g) = obs.try_lock() { g.record_processed(frame.clone()); }
            }
        }
    }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        let list = self.observers_clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_processed(frame.clone());
        }
        match frame {
            Frame::DataFrame(DataFrame::InputTextFrame(s)) => {
                let out = Frame::DataFrame(DataFrame::OutputTextFrame(s));
                let list2 = self.observers_clone();
                for obs in list2.iter() {
                    let mut g = obs.lock().await; g.record_emitted(out.clone());
                }
                Some(out)
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OutputProcessor {
    observers: Arc<StdMutex<Vec<Arc<Mutex<Observer>>>>>,
}

impl OutputProcessor {
    pub fn new() -> Self { Self { observers: Arc::new(StdMutex::new(Vec::new())) } }
    pub fn add_observer(&self, observer: Arc<Mutex<Observer>>) { if let Ok(mut v) = self.observers.lock() { v.push(observer); } }
    fn observers_clone(&self) -> Vec<Arc<Mutex<Observer>>> {
        self.observers.lock().map(|v| v.clone()).unwrap_or_default()
    }
    pub(crate) fn handle_system_sync(&self, frame: Frame) {
        if let Ok(list) = self.observers.lock() {
            for obs in list.iter() {
                if let Ok(mut g) = obs.try_lock() { g.record_processed(frame.clone()); }
            }
        }
    }
    pub async fn handle(&self, frame: Frame) -> Option<Frame> {
        let list = self.observers_clone();
        for obs in list.iter() {
            let mut g = obs.lock().await; g.record_processed(frame.clone());
        }
        Some(frame)
    }
}


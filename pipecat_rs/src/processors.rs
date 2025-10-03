use std::sync::Arc;
use tokio::sync::Mutex;
use crate::observer::Observer;

pub struct InputProcessor {
    observers: Vec<Arc<Mutex<Observer>>>,
}

impl InputProcessor {
    pub fn new() -> Self { Self { observers: Vec::new() } }
    pub fn add_observer(&self, _observer: Arc<Mutex<Observer>>) { /* todo! in compiler phase */ }
}

pub struct EchoProcessor {
    observers: Vec<Arc<Mutex<Observer>>>,
}

impl EchoProcessor {
    pub fn new() -> Self { Self { observers: Vec::new() } }
    pub fn add_observer(&self, _observer: Arc<Mutex<Observer>>) { /* stub */ }
}

pub struct OutputProcessor {
    observers: Vec<Arc<Mutex<Observer>>>,
}

impl OutputProcessor {
    pub fn new() -> Self { Self { observers: Vec::new() } }
    pub fn add_observer(&self, _observer: Arc<Mutex<Observer>>) { /* stub */ }
}


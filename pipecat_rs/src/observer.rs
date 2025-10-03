use pipecat_rs_locked::frame::Frame;

pub struct Observer {
    processed_frames: Vec<Frame>,
    emitted_frames: Vec<Frame>,
}

impl Observer {
    pub fn new() -> Self {
        Self {
            processed_frames: Vec::new(),
            emitted_frames: Vec::new(),
        }
    }

    pub fn processed_contains<F>(&self, predicate: F) -> bool
    where
        F: Fn(Frame) -> bool,
    {
        self.processed_frames.iter().cloned().any(predicate)
    }

    pub fn emitted_contains<F>(&self, predicate: F) -> bool
    where
        F: Fn(Frame) -> bool,
    {
        self.emitted_frames.iter().cloned().any(predicate)
    }
}


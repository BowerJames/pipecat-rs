use crate::processors::{InputProcessor, EchoProcessor, OutputProcessor};

pub enum AnyProcessor {
    Input(InputProcessor),
    Echo(EchoProcessor),
    Output(OutputProcessor),
}

pub struct Pipeline {
    processors: Vec<AnyProcessor>,
}

impl Pipeline {
    pub fn new() -> Self { Self { processors: Vec::new() } }

    pub fn add_processor<T>(&self, _processor: T)
    where
        T: Into<AnyProcessor>
    {
        // compiler-driven stub; no-op
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


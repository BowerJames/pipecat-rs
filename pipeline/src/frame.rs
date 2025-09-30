use uuid::Uuid;
use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameMeta {
    pub id: Uuid,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameDirection {
    Upstream,
    Downstream,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    System(FrameMeta, SystemFrame),
    Control(FrameMeta, ControlFrame),
    Data(FrameMeta, FrameDirection, DataFrame),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemFrame {
    Start,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFrame {
    Stop,
    End
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataFrame {
    InputText(String),
    OutputText(String),
}








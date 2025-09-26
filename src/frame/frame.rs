#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Upstream(FrameType),
    Downstream(FrameType),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameType {
    Heartbeat
}
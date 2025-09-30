use tokio::sync::mpsc::{Sender, Receiver};
use tokio::task::JoinHandle;
use crate::frame::{Frame, SystemFrame, ControlFrame, DataFrame};
use crate::frame_processor::FrameProcessor;

pub struct PipelineComponent {
    index: usize,
    system_frame_receiver: Receiver<SystemFrame>,
    control_frame_receiver: Receiver<ControlFrame>,
    data_frame_receiver: Receiver<DataFrame>,
    // Sender into the pipeline router, tagged with our component index
    router_sender: Sender<(usize, Frame)>,
    frame_processor: Box<dyn FrameProcessor + Send + Sync + 'static>,
}

impl PipelineComponent {
    pub fn new(
        index: usize,
        system_frame_receiver: Receiver<SystemFrame>,
        control_frame_receiver: Receiver<ControlFrame>,
        data_frame_receiver: Receiver<DataFrame>,
        router_sender: Sender<(usize, Frame)>,
        frame_processor: Box<dyn FrameProcessor + Send + Sync + 'static>,
    ) -> Self {
        Self {
            index,
            system_frame_receiver,
            control_frame_receiver,
            data_frame_receiver,
            router_sender,
            frame_processor,
        }
    }

    pub fn run(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            // Local channel for processor outputs (plain Frame)
            let (proc_tx, mut proc_rx) = tokio::sync::mpsc::channel::<Frame>(64);

            // Forwarder task: forwards processor outputs to the pipeline with our index
            let router_sender = self.router_sender.clone();
            let index = self.index;
            let forwarder = tokio::spawn(async move {
                while let Some(frame) = proc_rx.recv().await {
                    // Ignore send errors if router is closed (shutdown)
                    let _ = router_sender.send((index, frame)).await;
                }
            });

            let mut system_rx = self.system_frame_receiver;
            let mut control_rx = self.control_frame_receiver;
            let mut data_rx = self.data_frame_receiver;
            let processor = &self.frame_processor;

            loop {
                // Drain priority queues: system -> control -> data
                let mut made_progress = false;

                while let Ok(sf) = system_rx.try_recv() {
                    processor.process_frame(Frame::System(super::frame::FrameMeta { id: uuid::Uuid::new_v4(), timestamp: tokio::time::Instant::now() }, sf), proc_tx.clone()).await;
                    made_progress = true;
                }

                while let Ok(cf) = control_rx.try_recv() {
                    processor.process_frame(Frame::Control(super::frame::FrameMeta { id: uuid::Uuid::new_v4(), timestamp: tokio::time::Instant::now() }, cf), proc_tx.clone()).await;
                    made_progress = true;
                }

                while let Ok(df) = data_rx.try_recv() {
                    // Default direction handling is deferred to pipeline; component only processes
                    // We tag direction at source only if needed by processors; leave as Downstream by default
                    processor.process_frame(Frame::Data(super::frame::FrameMeta { id: uuid::Uuid::new_v4(), timestamp: tokio::time::Instant::now() }, super::frame::FrameDirection::Downstream, df), proc_tx.clone()).await;
                    made_progress = true;
                }

                if made_progress {
                    // Yield to avoid busy loop
                    tokio::task::yield_now().await;
                    continue;
                }

                // Await on any of the three with biased priority
                tokio::select! {
                    biased;

                    Some(sf) = system_rx.recv() => {
                        processor.process_frame(Frame::System(super::frame::FrameMeta { id: uuid::Uuid::new_v4(), timestamp: tokio::time::Instant::now() }, sf), proc_tx.clone()).await;
                    }
                    Some(cf) = control_rx.recv() => {
                        processor.process_frame(Frame::Control(super::frame::FrameMeta { id: uuid::Uuid::new_v4(), timestamp: tokio::time::Instant::now() }, cf), proc_tx.clone()).await;
                    }
                    Some(df) = data_rx.recv() => {
                        processor.process_frame(Frame::Data(super::frame::FrameMeta { id: uuid::Uuid::new_v4(), timestamp: tokio::time::Instant::now() }, super::frame::FrameDirection::Downstream, df), proc_tx.clone()).await;
                    }
                    else => {
                        break; // all channels closed
                    }
                }
            }

            // Wait for forwarder to drain
            let _ = forwarder.await;
        })
    }
}
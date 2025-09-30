use tokio::sync::mpsc::{Sender, Receiver};
use tokio::task::JoinHandle;

use crate::frame::{Frame, FrameDirection, SystemFrame, ControlFrame, DataFrame};
use crate::frame_processor::FrameProcessor;
use crate::pipeline_component::PipelineComponent;

pub struct ComponentHandles {
    pub system_tx: Sender<SystemFrame>,
    pub control_tx: Sender<ControlFrame>,
    pub data_tx: Sender<DataFrame>,
}

pub struct Pipeline {
    components: Vec<ComponentHandles>,
    // Fan-in from components to router: (origin index, Frame)
    router_rx: Receiver<(usize, Frame)>,
    router_tx: Sender<(usize, Frame)>,
    tasks: Vec<JoinHandle<()>>,
}

impl Pipeline {
    pub fn new(
        frame_processors: Vec<Box<dyn FrameProcessor + Send + Sync + 'static>>,
    ) -> Self {
        let num = frame_processors.len();
        let mut components: Vec<ComponentHandles> = Vec::with_capacity(num);
        let mut tasks: Vec<JoinHandle<()>> = Vec::with_capacity(num);

        // Shared router channel (fan-in)
        let (router_tx, router_rx) = tokio::sync::mpsc::channel::<(usize, Frame)>(256);

        for (index, processor) in frame_processors.into_iter().enumerate() {
            // Per-component inbox channels
            let (system_tx, system_rx) = tokio::sync::mpsc::channel::<SystemFrame>(64);
            let (control_tx, control_rx) = tokio::sync::mpsc::channel::<ControlFrame>(64);
            let (data_tx, data_rx) = tokio::sync::mpsc::channel::<DataFrame>(64);

            let component = PipelineComponent::new(
                index,
                system_rx,
                control_rx,
                data_rx,
                router_tx.clone(),
                processor,
            );
            let handle = component.run();
            tasks.push(handle);
            components.push(ComponentHandles { system_tx, control_tx, data_tx });
        }

        Self { components, router_rx, router_tx, tasks }
    }

    pub async fn run(mut self) {
        while let Some((origin, frame)) = self.router_rx.recv().await {
            match frame {
                Frame::System(_, sf) => {
                    for ch in &self.components {
                        let _ = ch.system_tx.send(sf.clone()).await;
                    }
                }
                Frame::Control(_, cf) => {
                    for ch in &self.components {
                        let _ = ch.control_tx.send(cf.clone()).await;
                    }
                }
                Frame::Data(_, dir, df) => {
                    let target = match dir {
                        FrameDirection::Downstream => origin.checked_add(1),
                        FrameDirection::Upstream => origin.checked_sub(1),
                    };

                    if let Some(t) = target {
                        if let Some(ch) = self.components.get(t) {
                            let _ = ch.data_tx.send(df).await;
                        } else {
                            // Out of bounds: drop or surface externally
                        }
                    } else {
                        // Out of bounds (upstream from 0): drop or surface externally
                    }
                }
            }
        }

        // Router channel closed; attempt to join tasks (they should exit once senders drop)
        for task in self.tasks {
            let _ = task.await;
        }
    }

    pub fn broadcast_system(&self, sf: SystemFrame) {
        for ch in &self.components {
            let _ = ch.system_tx.try_send(sf.clone());
        }
    }

    pub fn broadcast_control(&self, cf: ControlFrame) {
        for ch in &self.components {
            let _ = ch.control_tx.try_send(cf.clone());
        }
    }

    pub async fn send_data(&self, index: usize, _dir: FrameDirection, df: DataFrame) {
        if let Some(ch) = self.components.get(index) {
            let _ = ch.data_tx.send(df).await;
        }
    }
}
use crate::frame_processor::frame_processor::{FrameProcessor, DefaultFrameProcessor};
use crate::frame::frame::Frame;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Pipeline {
    frame_processors: Vec<Arc<Mutex<Box<dyn FrameProcessor>>>>,
    source: Arc<Mutex<Box<dyn FrameProcessor>>>,
    sink: Arc<Mutex<Box<dyn FrameProcessor>>>,
    source_processed_count: Arc<Mutex<usize>>,
    sink_processed_count: Arc<Mutex<usize>>,
    processor_processed_counts: Arc<Mutex<Vec<usize>>>,
}

impl Pipeline {
    pub fn new(
        frame_processors: Vec<Box<dyn FrameProcessor>>,
    ) -> Self {
        let processors_wrapped: Vec<Arc<Mutex<Box<dyn FrameProcessor>>>> = frame_processors
            .into_iter()
            .map(|p| Arc::new(Mutex::new(p)))
            .collect();

        Self { 
            frame_processors: processors_wrapped,
            source: Arc::new(Mutex::new(Box::new(DefaultFrameProcessor::new()))),
            sink: Arc::new(Mutex::new(Box::new(DefaultFrameProcessor::new()))),
            source_processed_count: Arc::new(Mutex::new(0)),
            sink_processed_count: Arc::new(Mutex::new(0)),
            processor_processed_counts: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        // initialize per-processor counters
        {
            let mut counts = self.processor_processed_counts.lock().await;
            if counts.len() != self.frame_processors.len() {
                counts.clear();
                counts.resize(self.frame_processors.len(), 0);
            }
        }
        // Keep the pipeline running indefinitely
        loop {
            // Check for frames in source and forward Downstream frames to sink
            let frames_to_forward_down = {
                let source_guard = self.source.lock().await;
                if let Some(source_processor) = source_guard.as_any().downcast_ref::<DefaultFrameProcessor>() {
                    let all_frames = source_processor.get_recieved_frames();
                    let mut processed_count = self.source_processed_count.lock().await;
                    let new_frames = all_frames[*processed_count..].to_vec();
                    *processed_count = all_frames.len();
                    new_frames
                } else {
                    Vec::new()
                }
            };
            
            // Forward only Downstream frames from source to sink
            let downstream_from_source: Vec<Frame> = frames_to_forward_down
                .into_iter()
                .filter(|f| matches!(f, Frame::Downstream(_)))
                .collect();

            if !downstream_from_source.is_empty() {
                // to processors via process_frame
                for processor in &self.frame_processors {
                    let mut guard = processor.lock().await;
                    for frame in &downstream_from_source {
                        guard.process_frame(*frame).await?;
                    }
                }
                // to sink via push_frame (external arrival)
                let sink_guard = self.sink.lock().await;
                for frame in &downstream_from_source {
                    sink_guard.push_frame(*frame).await?;
                }
            }
            
            // Check for frames in sink and forward Upstream frames to source
            let frames_to_forward_up = {
                let sink_guard = self.sink.lock().await;
                if let Some(sink_processor) = sink_guard.as_any().downcast_ref::<DefaultFrameProcessor>() {
                    let all_frames = sink_processor.get_recieved_frames();
                    let mut processed_count = self.sink_processed_count.lock().await;
                    let new_frames = all_frames[*processed_count..].to_vec();
                    *processed_count = all_frames.len();
                    new_frames
                } else {
                    Vec::new()
                }
            };

            // Forward only Upstream frames from sink to source
            let upstream_from_sink: Vec<Frame> = frames_to_forward_up
                .into_iter()
                .filter(|f| matches!(f, Frame::Upstream(_)))
                .collect();

            if !upstream_from_sink.is_empty() {
                // to processors via process_frame
                for processor in &self.frame_processors {
                    let mut guard = processor.lock().await;
                    for frame in &upstream_from_sink {
                        guard.process_frame(*frame).await?;
                    }
                }
                // to source via push_frame (external arrival)
                let source_guard = self.source.lock().await;
                for frame in &upstream_from_sink {
                    source_guard.push_frame(*frame).await?;
                }
            }
            
            // For each processor: forward newly PUSHED frames by direction (ignore frames we injected via process_frame)
            for (idx, processor) in self.frame_processors.iter().enumerate() {
                let frames_from_processor = {
                    let guard = processor.lock().await;
                    if let Some(proc_impl) = guard.as_any().downcast_ref::<DefaultFrameProcessor>() {
                        let all_frames = proc_impl.get_pushed_frames();
                        let mut counts = self.processor_processed_counts.lock().await;
                        let processed_count = &mut counts[idx];
                        let new_frames = all_frames[*processed_count..].to_vec();
                        *processed_count = all_frames.len();
                        new_frames
                    } else {
                        Vec::new()
                    }
                };

                if frames_from_processor.is_empty() {
                    continue;
                }

                let downstream_frames: Vec<Frame> = frames_from_processor
                    .iter()
                    .copied()
                    .filter(|f| matches!(f, Frame::Downstream(_)))
                    .collect();
                let upstream_frames: Vec<Frame> = frames_from_processor
                    .iter()
                    .copied()
                    .filter(|f| matches!(f, Frame::Upstream(_)))
                    .collect();

                // Downstream from processor -> sink
                if !downstream_frames.is_empty() {
                    let sink_guard = self.sink.lock().await;
                    for frame in &downstream_frames {
                        sink_guard.push_frame(*frame).await?;
                    }
                }

                // Upstream from processor -> source
                if !upstream_frames.is_empty() {
                    let source_guard = self.source.lock().await;
                    for frame in &upstream_frames {
                        source_guard.push_frame(*frame).await?;
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    pub fn get_source(&self) -> Arc<Mutex<Box<dyn FrameProcessor>>> {
        Arc::clone(&self.source)
    }

    pub fn get_sink(&self) -> Arc<Mutex<Box<dyn FrameProcessor>>> {
        Arc::clone(&self.sink)
    }

    pub fn set_source(&mut self, source: Box<dyn FrameProcessor>) {
        self.source = Arc::new(Mutex::new(source));
    }

    pub fn set_sink(&mut self, sink: Box<dyn FrameProcessor>) {
        self.sink = Arc::new(Mutex::new(sink));
    }
    
    pub fn get_processor(&self, index: usize) -> Arc<Mutex<Box<dyn FrameProcessor>>> {
        Arc::clone(&self.frame_processors[index])
    }

    pub async fn get_source_frame_count(&self) -> u32 {
        let processor = self.source.lock().await;
        if let Some(processor) = processor.as_any().downcast_ref::<DefaultFrameProcessor>() {
            processor.get_frame_count()
        } else {
            0
        }
    }
    
    pub async fn get_sink_frame_count(&self) -> u32 {
        let processor = self.sink.lock().await;
        if let Some(processor) = processor.as_any().downcast_ref::<DefaultFrameProcessor>() {
            processor.get_frame_count()
        } else {
            0
        }
    }
}
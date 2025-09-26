use pipecat_rs::pipeline::pipeline::Pipeline;
use pipecat_rs::frame::frame::{Frame, FrameType};
use tokio::{spawn, time::{sleep, Duration}};

#[tokio::test]

async fn test_empy_pipeline_starts() {
    let pipeline = Pipeline::new(vec![]);
    let pipeline_task = spawn(async move { pipeline.run().await });
    sleep(Duration::from_secs(1)).await;
    assert!(!pipeline_task.is_finished());
    pipeline_task.abort();
}

#[tokio::test]
async fn test_new_pipeline_has_source_and_sink() {
    let pipeline = Pipeline::new(vec![]);
    let _source = pipeline.get_source();
    let _sink = pipeline.get_sink();
    let pipeline_task = spawn(async move { pipeline.run().await });
    sleep(Duration::from_secs(1)).await;
    pipeline_task.abort();
}

#[tokio::test]
async fn test_pipeline_pushes_frame_upstream() {
    let pipeline = Pipeline::new(vec![]);
    
    // Get references to source and sink before starting the pipeline
    let source = pipeline.get_source();
    let sink = pipeline.get_sink();
    
    // Start the pipeline
    let pipeline_task = spawn(async move { pipeline.run().await });
    
    // Give the pipeline a moment to start
    sleep(Duration::from_millis(50)).await;

    // Push frame Downstream via source
    let source_frame = Frame::Downstream(FrameType::Heartbeat);
    let source_ref = source.clone();
    let source_guard = source_ref.lock().await;
    
    source_guard.push_frame(source_frame.clone()).await.unwrap();
    drop(source_guard);
    
    // Give the pipeline time to process the frames
    sleep(Duration::from_millis(50)).await;

    // Check that the frame was received by the sink and source
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    let sink_frames = sink_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let source_ref = source.clone();
    let source_guard = source_ref.lock().await;
    let source_frames = source_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    assert_eq!(sink_frames, vec![source_frame]);
    assert_eq!(source_frames, vec![source_frame]);
    drop(sink_guard);
    drop(source_guard);

    pipeline_task.abort();
}

#[tokio::test]
async fn test_pipeline_pushes_frame_downstream() {
    let pipeline = Pipeline::new(vec![]);
    let source = pipeline.get_source();
    let sink = pipeline.get_sink();

    // Start the pipeline
    let pipeline_task = spawn(async move { pipeline.run().await });
    
    // Give the pipeline a moment to start
    sleep(Duration::from_millis(50)).await;

    // Push frame Upstream via sink
    let sink_frame = Frame::Upstream(FrameType::Heartbeat);
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    sink_guard.push_frame(sink_frame.clone()).await.unwrap();
    drop(sink_guard);

    // Give the pipeline time to process the frames
    sleep(Duration::from_millis(50)).await;

    // Check that the frame was received by the sink and source
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    let sink_frames = sink_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let source_ref = source.clone();
    let source_guard = source_ref.lock().await;
    let source_frames = source_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    assert_eq!(sink_frames, vec![sink_frame]);
    assert_eq!(source_frames, vec![sink_frame]);
    drop(sink_guard);
    drop(source_guard);
    pipeline_task.abort();
}
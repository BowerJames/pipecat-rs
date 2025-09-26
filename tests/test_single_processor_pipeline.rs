use pipecat_rs::pipeline::pipeline::Pipeline;
use pipecat_rs::frame_processor::frame_processor::{DefaultFrameProcessor};
use pipecat_rs::frame::frame::{Frame, FrameType};
use tokio::{spawn, time::{sleep, Duration}};


#[tokio::test]
async fn test_single_processor_pipeline_push_to_source() {
    let pipeline = Pipeline::new(vec![Box::new(DefaultFrameProcessor::new())]);

    let source = pipeline.get_source();
    let sink = pipeline.get_sink();
    let default_processor = pipeline.get_processor(0).clone();
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

    // Check that the frame was received by the source, default processor and sink
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    let sink_frames = sink_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let default_processor_ref = default_processor.clone();
    let default_processor_guard = default_processor_ref.lock().await;
    let default_processor_frames = default_processor_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let source_ref = source.clone();
    let source_guard = source_ref.lock().await;
    let source_frames = source_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();

    assert_eq!(sink_frames, vec![source_frame]);
    assert_eq!(default_processor_frames, vec![source_frame]);
    assert_eq!(source_frames, vec![source_frame]);
    drop(sink_guard);
    drop(default_processor_guard);
    drop(source_guard);
    pipeline_task.abort();
}

#[tokio::test]
async fn test_single_processor_pipeline_push_to_sink() {
    let pipeline = Pipeline::new(vec![Box::new(DefaultFrameProcessor::new())]);

    let source = pipeline.get_source();
    let sink = pipeline.get_sink();
    let default_processor = pipeline.get_processor(0).clone();

    // Start the pipeline
    let pipeline_task = spawn(async move { pipeline.run().await });

    // Give the pipeline a moment to start
    sleep(Duration::from_millis(50)).await;

    // Push frame Upstream via sink
    let pushed_frame = Frame::Upstream(FrameType::Heartbeat);
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    sink_guard.push_frame(pushed_frame.clone()).await.unwrap();
    drop(sink_guard);

    // Give the pipeline time to process the frames
    sleep(Duration::from_millis(50)).await;

    // Check that the frame was received by the default processor and sink and source
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    let sink_frames = sink_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let default_processor_ref = default_processor.clone();
    let default_processor_guard = default_processor_ref.lock().await;
    let default_processor_frames = default_processor_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let source_ref = source.clone();
    let source_guard = source_ref.lock().await;
    let source_frames = source_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();

    assert_eq!(sink_frames, vec![pushed_frame]);
    assert_eq!(default_processor_frames, vec![pushed_frame]);
    assert_eq!(source_frames, vec![pushed_frame]);

    drop(sink_guard);
    drop(default_processor_guard);
    drop(source_guard);
    pipeline_task.abort();
}

#[tokio::test]
async fn test_single_processor_pipeline_push_to_default_processor_downstream() {
    let pipeline = Pipeline::new(vec![Box::new(DefaultFrameProcessor::new())]);

    let source = pipeline.get_source();
    let sink = pipeline.get_sink();
    let default_processor = pipeline.get_processor(0).clone();

    // Start the pipeline
    let pipeline_task = spawn(async move { pipeline.run().await });

    // Give the pipeline a moment to start
    sleep(Duration::from_millis(50)).await;

    // Push frame Downstream via source
    let pushed_frame = Frame::Downstream(FrameType::Heartbeat);
    let default_processor_ref = default_processor.clone();
    let default_processor_guard = default_processor_ref.lock().await;
    default_processor_guard.push_frame(pushed_frame.clone()).await.unwrap();
    drop(default_processor_guard);

    // Give the pipeline time to process the frames
    sleep(Duration::from_millis(50)).await;

    // Check that the frame was received by the default processor and sink but not the source
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    let sink_frames = sink_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let default_processor_ref = default_processor.clone();
    let default_processor_guard = default_processor_ref.lock().await;
    let default_processor_frames = default_processor_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let source_ref = source.clone();
    let source_guard = source_ref.lock().await;
    let source_frames = source_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();

    assert_eq!(sink_frames, vec![pushed_frame]);
    assert_eq!(default_processor_frames, vec![pushed_frame]);
    assert_eq!(source_frames, vec![]);

    drop(sink_guard);
    drop(default_processor_guard);
    drop(source_guard);
    pipeline_task.abort();
}

#[tokio::test]
async fn test_single_processor_pipeline_push_to_default_processor_upstream() {
    let pipeline = Pipeline::new(vec![Box::new(DefaultFrameProcessor::new())]);

    let source = pipeline.get_source();
    let sink = pipeline.get_sink();
    let default_processor = pipeline.get_processor(0).clone();

    // Start the pipeline
    let pipeline_task = spawn(async move { pipeline.run().await });

    // Give the pipeline a moment to start
    sleep(Duration::from_millis(50)).await;

    // Push frame Upstream via sink
    let pushed_frame = Frame::Upstream(FrameType::Heartbeat);
    let default_processor_ref = default_processor.clone();
    let default_processor_guard = default_processor_ref.lock().await;
    default_processor_guard.push_frame(pushed_frame.clone()).await.unwrap();
    drop(default_processor_guard);

    // Give the pipeline time to process the frames
    sleep(Duration::from_millis(50)).await;

    // Check that the frame was received by the default processor and source but not the sink
    let sink_ref = sink.clone();
    let sink_guard = sink_ref.lock().await;
    let sink_frames = sink_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let default_processor_ref = default_processor.clone();
    let default_processor_guard = default_processor_ref.lock().await;
    let default_processor_frames = default_processor_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();
    let source_ref = source.clone();
    let source_guard = source_ref.lock().await;
    let source_frames = source_guard.as_any().downcast_ref::<pipecat_rs::frame_processor::frame_processor::DefaultFrameProcessor>().unwrap().get_recieved_frames();

    assert_eq!(sink_frames, vec![]);
    assert_eq!(default_processor_frames, vec![pushed_frame]);
    assert_eq!(source_frames, vec![pushed_frame]);

    drop(sink_guard);
    drop(default_processor_guard);
    drop(source_guard);
    pipeline_task.abort();
}


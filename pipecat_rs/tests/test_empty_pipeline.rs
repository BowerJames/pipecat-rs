use pipecat_rs::Pipeline;

#[tokio::test]
async fn test_empty_pipeline_builds() {
    let processors = vec![];
    let pipeline = Pipeline::new(processors);
    assert!(pipeline.is_ok());
}

#[tokio::test]
async fn test_empty_pipeline_cant_serve() {
    let processors = vec![];
    let pipeline = Pipeline::new(processors);
    pipeline.serve().await.unwrap_err();
}
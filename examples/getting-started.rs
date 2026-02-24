use observation_tools::{group, observe, with_execution, ClientBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create a client pointing at a running observation-tools server
    let client = ClientBuilder::new()
        .base_url("http://localhost:3000")
        .build()?;

    // 2. Begin an execution — this is the root scope for all observations
    let execution = client
        .begin_execution("getting-started")?
        .wait_for_upload()
        .await?;
    println!("Execution URL: {}", execution.url());

    // 3. Use with_execution to set the context for observe! calls
    with_execution(execution, async {
        let user_message = "What is the topic of this document?";
        observe!("user_message").payload(user_message);

        let document_content = load_document_content().await;
        observe!("document_content").serde(&document_content);

        let api_request = serde_json::json!({
            "message": user_message,
            "document": document_content,
        });
        observe!("api_call").serde(&api_request);
        let api_response = call_api(api_request).await;
        observe!("api_response").serde(&api_response);

        let group = group!("processing_steps").build().into_handle();
        observe!("hello").group(&group).payload("Hello, world!");
    })
    .await;

    // 4. Shut down the client to flush all pending uploads
    client.shutdown().await?;
    println!("Done! Open the execution URL above to view your observations.");

    Ok(())
}

async fn load_document_content() -> serde_json::Value {
    serde_json::json!({
        "title": "Rust Programming Guide",
        "content": "Rust offers memory safety and fast performance.",
    })
}

async fn call_api(_request: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "message": "The main theme of the document is Rust programming.",
    })
}


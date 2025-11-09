use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use shared::{get_db_client, upsert_statistics, SessionStatistics};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
struct ToolSuccessFail {
    success: i64,
    failed: i64,
}

#[derive(Debug, Deserialize)]
struct StatisticsRequest {
    session_id: String,
    provider: String,
    model: String,
    input_tokens: i64,
    output_tokens: i64,
    waiting_for_human_ms: i64,
    ai_processing_ms: i64,
    tool_execution_ms: i64,
    tool_calls: HashMap<String, i64>, // Tool name -> count
    tool_success_fail: HashMap<String, ToolSuccessFail>, // Tool name -> success/fail counts
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    info!("Received statistics request");

    // Parse request body
    let body = event.body();
    let req: StatisticsRequest = match body {
        Body::Text(text) => serde_json::from_str(text)?,
        Body::Binary(bytes) => serde_json::from_slice(bytes)?,
        Body::Empty => {
            return Ok(Response::builder()
                .status(400)
                .body(Body::Text("Request body is required".to_string()))?);
        }
    };

    // Convert tool calls to JSON string
    let tool_calls_json = serde_json::to_string(&req.tool_calls)?;
    let tool_success_fail_json = serde_json::to_string(&req.tool_success_fail)?;

    let stats = SessionStatistics {
        session_id: req.session_id.clone(),
        provider: req.provider,
        model: req.model,
        input_tokens: req.input_tokens,
        output_tokens: req.output_tokens,
        waiting_for_human_ms: req.waiting_for_human_ms,
        ai_processing_ms: req.ai_processing_ms,
        tool_execution_ms: req.tool_execution_ms,
        tool_calls_json,
        tool_success_fail_json,
    };

    // Connect to database and upsert statistics
    let client = get_db_client().await?;
    upsert_statistics(&client, &stats).await?;

    info!("Successfully upserted statistics for session: {}", req.session_id);

    // Return success response
    let response = serde_json::json!({
        "success": true,
        "session_id": req.session_id,
        "message": "Statistics recorded successfully"
    });

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::Text(serde_json::to_string(&response)?))?)

}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::Deserialize;
use shared::{get_db_client, insert_bug_report};
use tracing::info;

#[derive(Debug, Deserialize)]
struct BugReportRequest {
    description: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    info!("Received bug report request");

    // Parse request body
    let body = event.body();
    let req: BugReportRequest = match body {
        Body::Text(text) => serde_json::from_str(text)?,
        Body::Binary(bytes) => serde_json::from_slice(bytes)?,
        Body::Empty => {
            return Ok(Response::builder()
                .status(400)
                .body(Body::Text("Request body is required".to_string()))?);
        }
    };

    // Validate description is not empty
    if req.description.trim().is_empty() {
        return Ok(Response::builder()
            .status(400)
            .body(Body::Text("Bug description cannot be empty".to_string()))?);
    }

    // Connect to database and insert bug report
    let client = get_db_client().await?;
    let bug_id = insert_bug_report(&client, &req.description).await?;

    info!("Successfully inserted bug report with ID: {}", bug_id);

    // Return success response
    let response = serde_json::json!({
        "success": true,
        "bug_id": bug_id,
        "message": "Bug report submitted successfully"
    });

    Ok(Response::builder()
        .status(201)
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

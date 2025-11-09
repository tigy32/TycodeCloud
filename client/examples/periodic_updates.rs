use std::time::Duration;
use tokio::time::sleep;
use tycode_client::{SessionStatisticsBuilder, TycodeClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_endpoint = std::env::var("TYCODE_API_ENDPOINT")
        .unwrap_or_else(|_| "https://your-api.execute-api.us-west-2.amazonaws.com".to_string());

    let client = TycodeClient::new(api_endpoint);
    let session_id = "periodic-session-001";

    println!("Starting periodic updates (upsert by session_id)...\n");

    // Simulate 5 periodic updates
    for iteration in 1..=5 {
        let stats = SessionStatisticsBuilder::new(session_id, "anthropic", "claude-sonnet-4")
            .tokens(iteration * 100, iteration * 50)
            .timing(iteration * 1000, iteration * 200, iteration * 100)
            .add_tool_call("Read", iteration)
            .add_tool_call("Write", iteration / 2)
            .add_tool_stats("Read", iteration, 0)
            .add_tool_stats("Write", iteration / 2, iteration % 2)
            .build();

        match client.report_statistics(&stats).await {
            Ok(response) => {
                println!(
                    "Update {}/5: {} (tokens: {}/{})",
                    iteration,
                    response.message,
                    stats.input_tokens,
                    stats.output_tokens
                );
            }
            Err(e) => {
                eprintln!("Update {}/5 failed: {}", iteration, e);
            }
        }

        if iteration < 5 {
            println!("Waiting 2 seconds before next update...\n");
            sleep(Duration::from_secs(2)).await;
        }
    }

    println!("\nAll updates completed!");

    Ok(())
}

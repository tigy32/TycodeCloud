use tycode_client::{SessionStatisticsBuilder, TycodeClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace with your actual API endpoint
    let api_endpoint = std::env::var("TYCODE_API_ENDPOINT")
        .unwrap_or_else(|_| "https://your-api.execute-api.us-west-2.amazonaws.com".to_string());

    let client = TycodeClient::new(api_endpoint);

    // Report session statistics
    println!("Reporting session statistics...");
    let stats = SessionStatisticsBuilder::new("example-session-001", "anthropic", "claude-sonnet-4")
        .tokens(1500, 800)
        .timing(45000, 3200, 1800)
        .add_tool_call("Read", 5)
        .add_tool_call("Write", 3)
        .add_tool_call("Bash", 2)
        .add_tool_stats("Read", 5, 0)
        .add_tool_stats("Write", 2, 1)
        .add_tool_stats("Bash", 2, 0)
        .build();

    let response = client.report_statistics(&stats).await?;
    println!("✓ Statistics reported: {}", response.message);
    println!("  Session ID: {}", response.session_id);

    // Report a bug
    println!("\nReporting a bug...");
    let bug_response = client
        .report_bug("Example bug report from Rust client")
        .await?;
    println!("✓ Bug reported: {}", bug_response.message);
    println!("  Bug ID: {}", bug_response.bug_id);

    Ok(())
}

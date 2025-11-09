# Tycode Client

Rust client library for the Tycode Cloud API.

## Installation

### Option 1: Git Dependency (Recommended for Private Repos)

Add to your `Cargo.toml`:

```toml
[dependencies]
tycode-client = { git = "https://github.com/tigy32/TycodeCloud", branch = "main" }
```

Or pin to a specific commit:

```toml
[dependencies]
tycode-client = { git = "https://github.com/tigy32/TycodeCloud", rev = "abc123" }
```

### Option 2: Path Dependency (Local Development)

```toml
[dependencies]
tycode-client = { path = "../TycodeCloud/client" }
```

### Option 3: Workspace Member

If tycode is a Rust workspace, add to your workspace `Cargo.toml`:

```toml
[workspace]
members = ["tycode", "TycodeCloud/client"]
```

Then in your project:

```toml
[dependencies]
tycode-client = { path = "../TycodeCloud/client" }
```

## Usage

### Basic Example

```rust
use tycode_client::{TycodeClient, SessionStatisticsBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = TycodeClient::new("https://your-api-endpoint.amazonaws.com");

    // Build statistics using the builder pattern
    let stats = SessionStatisticsBuilder::new(
        "session-123",
        "anthropic",
        "claude-sonnet-4"
    )
    .tokens(1500, 800)
    .timing(45000, 3200, 1800)
    .add_tool_call("Read", 5)
    .add_tool_call("Write", 3)
    .add_tool_call("Bash", 2)
    .add_tool_stats("Read", 5, 0)
    .add_tool_stats("Write", 2, 1)
    .add_tool_stats("Bash", 2, 0)
    .build();

    // Report statistics
    let response = client.report_statistics(&stats).await?;
    println!("Stats reported: {}", response.message);

    Ok(())
}
```

### Reporting Bugs

```rust
use tycode_client::TycodeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TycodeClient::new("https://your-api-endpoint.amazonaws.com");

    let response = client.report_bug("Something went wrong!").await?;
    println!("Bug reported with ID: {}", response.bug_id);

    Ok(())
}
```

### Manual Construction

```rust
use tycode_client::{TycodeClient, SessionStatistics, ToolSuccessFail};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TycodeClient::new("https://your-api-endpoint.amazonaws.com");

    let mut tool_calls = HashMap::new();
    tool_calls.insert("Read".to_string(), 5);
    tool_calls.insert("Write".to_string(), 3);

    let mut tool_success_fail = HashMap::new();
    tool_success_fail.insert("Read".to_string(), ToolSuccessFail::new(5, 0));
    tool_success_fail.insert("Write".to_string(), ToolSuccessFail::new(2, 1));

    let stats = SessionStatistics {
        session_id: "session-123".to_string(),
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4".to_string(),
        input_tokens: 1500,
        output_tokens: 800,
        waiting_for_human_ms: 45000,
        ai_processing_ms: 3200,
        tool_execution_ms: 1800,
        tool_calls,
        tool_success_fail,
    };

    client.report_statistics(&stats).await?;

    Ok(())
}
```

### Periodic Reporting

```rust
use tycode_client::{TycodeClient, SessionStatisticsBuilder};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TycodeClient::new("https://your-api-endpoint.amazonaws.com");
    let session_id = "my-session-id";

    // Simulate periodic updates (upserts by session_id)
    for i in 1..=5 {
        let stats = SessionStatisticsBuilder::new(
            session_id,
            "anthropic",
            "claude-sonnet-4"
        )
        .tokens(i * 100, i * 50)
        .timing(i * 1000, i * 200, i * 100)
        .add_tool_call("Read", i)
        .add_tool_stats("Read", i, 0)
        .build();

        client.report_statistics(&stats).await?;
        println!("Update {} sent", i);

        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
```

## Error Handling

```rust
use tycode_client::{TycodeClient, TycodeError};

#[tokio::main]
async fn main() {
    let client = TycodeClient::new("https://your-api-endpoint.amazonaws.com");

    match client.report_bug("Test bug").await {
        Ok(response) => println!("Bug ID: {}", response.bug_id),
        Err(TycodeError::RequestFailed(e)) => eprintln!("Network error: {}", e),
        Err(TycodeError::ApiError(e)) => eprintln!("API error: {}", e),
    }
}
```

## Features

- **Type-safe API**: Strongly typed request/response structures
- **Builder pattern**: Convenient API for constructing statistics
- **Async/await**: Built on tokio and reqwest
- **Error handling**: Custom error types with thiserror
- **Idempotent updates**: Statistics are upserted by session_id
- **Lightweight**: Minimal dependencies

## API Reference

### `TycodeClient`

- `new(base_url)` - Create a new client
- `report_statistics(&stats)` - Report session statistics (idempotent)
- `report_bug(description)` - Submit a bug report

### `SessionStatisticsBuilder`

- `new(session_id, provider, model)` - Start building statistics
- `tokens(input, output)` - Set token counts
- `timing(waiting, ai_processing, tool_execution)` - Set timing in milliseconds
- `add_tool_call(name, count)` - Add tool call count
- `add_tool_stats(name, success, failed)` - Add tool success/fail stats
- `build()` - Build the SessionStatistics

### Types

- `SessionStatistics` - Session usage statistics
- `ToolSuccessFail` - Tool success/failure counts
- `StatisticsResponse` - API response for statistics
- `BugReportResponse` - API response for bug reports
- `TycodeError` - Error type for client operations

## License

MIT

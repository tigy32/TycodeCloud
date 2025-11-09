use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config};
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

/// Session statistics model
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub session_id: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub waiting_for_human_ms: i64,
    pub ai_processing_ms: i64,
    pub tool_execution_ms: i64,
    pub tool_calls_json: String, // JSON string of tool call counts
    pub tool_success_fail_json: String, // JSON string of tool success/fail rates
}

/// Bug report model
#[derive(Debug, Serialize, Deserialize)]
pub struct BugReport {
    pub bug_id: Uuid,
    pub description: String,
}

/// Generate DSQL authentication token
async fn generate_dsql_token(hostname: &str, region: &str) -> Result<String> {
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let config = Config::builder()
        .hostname(hostname)
        .region(aws_config::Region::new(region.to_string()))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build DSQL auth config: {}", e))?;

    let signer = AuthTokenGenerator::new(config);

    // Use admin auth token for database connection
    let token = signer
        .db_connect_admin_auth_token(&sdk_config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate DSQL auth token: {}", e))?;

    Ok(token.to_string())
}

/// Database connection helper
pub async fn get_db_client() -> Result<Client> {
    let db_host = std::env::var("DB_HOST").context("DB_HOST not set")?;
    let db_name = std::env::var("DB_NAME").context("DB_NAME not set")?;
    let region = std::env::var("AWS_REGION").context("AWS_REGION not set")?;

    // Generate DSQL auth token
    let token = generate_dsql_token(&db_host, &region).await?;

    let connection_string = format!(
        "host={} dbname={} user=admin password={} sslmode=require",
        db_host, db_name, token
    );

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .context("Failed to connect to database")?;

    // Spawn connection in background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    Ok(client)
}

/// Upsert session statistics
pub async fn upsert_statistics(client: &Client, stats: &SessionStatistics) -> Result<()> {
    let query = r#"
        INSERT INTO session_statistics (
            session_id, provider, model, input_tokens, output_tokens,
            waiting_for_human_ms, ai_processing_ms, tool_execution_ms, tool_calls_json, tool_success_fail_json
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (session_id)
        DO UPDATE SET
            provider = EXCLUDED.provider,
            model = EXCLUDED.model,
            input_tokens = EXCLUDED.input_tokens,
            output_tokens = EXCLUDED.output_tokens,
            waiting_for_human_ms = EXCLUDED.waiting_for_human_ms,
            ai_processing_ms = EXCLUDED.ai_processing_ms,
            tool_execution_ms = EXCLUDED.tool_execution_ms,
            tool_calls_json = EXCLUDED.tool_calls_json,
            tool_success_fail_json = EXCLUDED.tool_success_fail_json,
            updated_at = CURRENT_TIMESTAMP
    "#;

    client
        .execute(
            query,
            &[
                &stats.session_id,
                &stats.provider,
                &stats.model,
                &stats.input_tokens,
                &stats.output_tokens,
                &stats.waiting_for_human_ms,
                &stats.ai_processing_ms,
                &stats.tool_execution_ms,
                &stats.tool_calls_json,
                &stats.tool_success_fail_json,
            ],
        )
        .await
        .context("Failed to upsert statistics")?;

    Ok(())
}

/// Insert bug report
pub async fn insert_bug_report(client: &Client, description: &str) -> Result<Uuid> {
    let bug_id = Uuid::new_v4();

    let query = r#"
        INSERT INTO bug_reports (bug_id, description)
        VALUES ($1, $2)
    "#;

    client
        .execute(query, &[&bug_id, &description])
        .await
        .context("Failed to insert bug report")?;

    Ok(bug_id)
}

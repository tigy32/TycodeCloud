use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Client errors
#[derive(Error, Debug)]
pub enum TycodeError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),
}

pub type Result<T> = std::result::Result<T, TycodeError>;

/// Tool success/fail statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSuccessFail {
    pub success: i64,
    pub failed: i64,
}

impl ToolSuccessFail {
    pub fn new(success: i64, failed: i64) -> Self {
        Self { success, failed }
    }
}

/// Session statistics request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub session_id: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub waiting_for_human_ms: i64,
    pub ai_processing_ms: i64,
    pub tool_execution_ms: i64,
    pub tool_calls: HashMap<String, i64>,
    pub tool_success_fail: HashMap<String, ToolSuccessFail>,
}

/// Statistics API response
#[derive(Debug, Deserialize)]
pub struct StatisticsResponse {
    pub success: bool,
    pub session_id: String,
    pub message: String,
}

/// Bug report response
#[derive(Debug, Deserialize)]
pub struct BugReportResponse {
    pub success: bool,
    pub bug_id: Uuid,
    pub message: String,
}

/// Tycode Cloud API client
#[derive(Debug, Clone)]
pub struct TycodeClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl TycodeClient {
    /// Create a new client with the API base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Report session statistics (idempotent by session_id)
    pub async fn report_statistics(&self, stats: &SessionStatistics) -> Result<StatisticsResponse> {
        let url = format!("{}/statistics", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .json(stats)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TycodeError::ApiError(error_text));
        }

        let result = response.json::<StatisticsResponse>().await?;
        Ok(result)
    }

    /// Submit a bug report
    pub async fn report_bug(&self, description: impl Into<String>) -> Result<BugReportResponse> {
        let url = format!("{}/bugs", self.base_url);

        let payload = serde_json::json!({
            "description": description.into()
        });

        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TycodeError::ApiError(error_text));
        }

        let result = response.json::<BugReportResponse>().await?;
        Ok(result)
    }
}

/// Builder for creating SessionStatistics
pub struct SessionStatisticsBuilder {
    session_id: String,
    provider: String,
    model: String,
    input_tokens: i64,
    output_tokens: i64,
    waiting_for_human_ms: i64,
    ai_processing_ms: i64,
    tool_execution_ms: i64,
    tool_calls: HashMap<String, i64>,
    tool_success_fail: HashMap<String, ToolSuccessFail>,
}

impl SessionStatisticsBuilder {
    pub fn new(session_id: impl Into<String>, provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            provider: provider.into(),
            model: model.into(),
            input_tokens: 0,
            output_tokens: 0,
            waiting_for_human_ms: 0,
            ai_processing_ms: 0,
            tool_execution_ms: 0,
            tool_calls: HashMap::new(),
            tool_success_fail: HashMap::new(),
        }
    }

    pub fn tokens(mut self, input: i64, output: i64) -> Self {
        self.input_tokens = input;
        self.output_tokens = output;
        self
    }

    pub fn timing(mut self, waiting: i64, ai_processing: i64, tool_execution: i64) -> Self {
        self.waiting_for_human_ms = waiting;
        self.ai_processing_ms = ai_processing;
        self.tool_execution_ms = tool_execution;
        self
    }

    pub fn add_tool_call(mut self, tool_name: impl Into<String>, count: i64) -> Self {
        self.tool_calls.insert(tool_name.into(), count);
        self
    }

    pub fn add_tool_stats(mut self, tool_name: impl Into<String>, success: i64, failed: i64) -> Self {
        self.tool_success_fail.insert(tool_name.into(), ToolSuccessFail::new(success, failed));
        self
    }

    pub fn build(self) -> SessionStatistics {
        SessionStatistics {
            session_id: self.session_id,
            provider: self.provider,
            model: self.model,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            waiting_for_human_ms: self.waiting_for_human_ms,
            ai_processing_ms: self.ai_processing_ms,
            tool_execution_ms: self.tool_execution_ms,
            tool_calls: self.tool_calls,
            tool_success_fail: self.tool_success_fail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let stats = SessionStatisticsBuilder::new("session-123", "anthropic", "claude-sonnet-4")
            .tokens(1000, 500)
            .timing(30000, 2000, 1000)
            .add_tool_call("Read", 5)
            .add_tool_call("Write", 3)
            .add_tool_stats("Read", 5, 0)
            .add_tool_stats("Write", 2, 1)
            .build();

        assert_eq!(stats.session_id, "session-123");
        assert_eq!(stats.input_tokens, 1000);
        assert_eq!(stats.tool_calls.get("Read"), Some(&5));
        assert_eq!(stats.tool_success_fail.get("Read").unwrap().success, 5);
    }
}

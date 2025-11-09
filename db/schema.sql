-- Tycode Statistics and Bug Reporting Database Schema
-- For Aurora DSQL

-- Session statistics table
CREATE TABLE IF NOT EXISTS session_statistics (
    session_id VARCHAR(255) PRIMARY KEY,
    provider VARCHAR(100) NOT NULL,
    model VARCHAR(100) NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    waiting_for_human_ms BIGINT NOT NULL DEFAULT 0,
    ai_processing_ms BIGINT NOT NULL DEFAULT 0,
    tool_execution_ms BIGINT NOT NULL DEFAULT 0,
    tool_calls_json TEXT NOT NULL, -- JSON serialized as string
    tool_success_fail_json TEXT NOT NULL, -- JSON serialized as string: {"ToolName": {"success": N, "failed": M}}
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Bug reports table
CREATE TABLE IF NOT EXISTS bug_reports (
    bug_id UUID PRIMARY KEY,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_session_stats_created_at ON session_statistics(created_at);
CREATE INDEX IF NOT EXISTS idx_session_stats_provider ON session_statistics(provider);
CREATE INDEX IF NOT EXISTS idx_bug_reports_created_at ON bug_reports(created_at);

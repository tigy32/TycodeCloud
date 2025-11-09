# Database Schema

This directory contains the database schema for the Tycode API.

## Schema Files

- `schema.sql` - Complete database schema for Aurora DSQL

## Tables

### session_statistics
Stores usage statistics for Tycode sessions. Updated via upsert (idempotent by session_id).

- `session_id` (VARCHAR, PRIMARY KEY) - Unique session identifier
- `provider` (VARCHAR) - AI provider name
- `model` (VARCHAR) - Model name
- `input_tokens` (BIGINT) - Total input tokens used
- `output_tokens` (BIGINT) - Total output tokens used
- `waiting_for_human_ms` (BIGINT) - Time waiting for human input (milliseconds)
- `ai_processing_ms` (BIGINT) - AI processing time (milliseconds)
- `tool_execution_ms` (BIGINT) - Tool execution time (milliseconds)
- `tool_calls_json` (TEXT) - JSON string of tool call counts by name
- `tool_success_fail_json` (TEXT) - JSON string of tool success/fail rates by name (e.g., `{"Read": {"success": 5, "failed": 1}}`)
- `created_at` (TIMESTAMP) - Record creation timestamp
- `updated_at` (TIMESTAMP) - Last update timestamp

### bug_reports
Stores bug reports submitted by users.

- `bug_id` (UUID, PRIMARY KEY) - Unique bug identifier
- `description` (TEXT) - Bug description
- `created_at` (TIMESTAMP) - Report submission timestamp

## Running Migrations

The schema will be automatically created during Terraform deployment. To manually run:

```bash
psql -h <aurora-dsql-endpoint> -U <username> -d <database> -f schema.sql
```

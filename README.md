# Tycode Cloud API

A serverless API for Tycode statistics tracking and bug reporting, built with Rust Lambda functions and Aurora DSQL on AWS.

## Architecture

- **Language**: Rust
- **Compute**: AWS Lambda (ARM64)
- **Database**: Aurora DSQL
- **API Gateway**: HTTP API (cheaper than REST API)
- **Region**: us-west-2
- **Infrastructure**: Terraform

## API Endpoints

### POST /statistics
Records or updates session statistics for Tycode usage tracking.

**Request Body:**
```json
{
  "session_id": "unique-session-identifier",
  "provider": "anthropic",
  "model": "claude-sonnet-4",
  "input_tokens": 1500,
  "output_tokens": 800,
  "waiting_for_human_ms": 45000,
  "ai_processing_ms": 3200,
  "tool_execution_ms": 1800,
  "tool_calls": {
    "Read": 5,
    "Write": 3,
    "Bash": 2
  },
  "tool_success_fail": {
    "Read": {"success": 5, "failed": 0},
    "Write": {"success": 2, "failed": 1},
    "Bash": {"success": 2, "failed": 0}
  }
}
```

**Response:**
```json
{
  "success": true,
  "session_id": "unique-session-identifier",
  "message": "Statistics recorded successfully"
}
```

### POST /bugs
Submits a bug report.

**Request Body:**
```json
{
  "description": "Detailed description of the bug encountered"
}
```

**Response:**
```json
{
  "success": true,
  "bug_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Bug report submitted successfully"
}
```

## Prerequisites

- **Rust** (latest stable): Install from [rustup.rs](https://rustup.rs/)
- **cargo-lambda**: For building Lambda functions
- **Terraform** (>= 1.0): For infrastructure deployment
- **AWS CLI**: Configured with appropriate credentials
- **AWS Account**: With permissions to create Lambda, API Gateway, RDS, IAM resources

## Setup and Deployment

### 1. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-lambda
pip3 install cargo-lambda

# Install Terraform
# macOS
brew install terraform

# Linux
# See https://developer.hashicorp.com/terraform/downloads
```

### 2. Build Lambda Functions

```bash
# Build both Lambda functions for ARM64
./build.sh
```

This creates deployment packages at:
- `target/lambda/statistics/bootstrap.zip`
- `target/lambda/bugs/bootstrap.zip`

### 3. Configure Terraform Variables (Optional)

The deployment uses default values, but you can customize by creating `terraform/terraform.tfvars`:

```hcl
aws_region   = "us-west-2"  # Default region
project_name = "tycode-api" # Default project name
```

**Note**: Aurora DSQL uses IAM authentication (no passwords needed!)

### 4. Deploy Infrastructure

```bash
cd terraform

# Initialize Terraform
terraform init

# Preview changes
terraform plan

# Deploy
terraform apply
```

### 5. Initialize Database

After Terraform deployment, connect to the Aurora DSQL cluster and run the schema:

```bash
# Get the database endpoint from Terraform output
terraform output database_endpoint

# Run the schema
psql -h <database-endpoint> -U admin -d tycode -f ../db/schema.sql
```

### 6. Test the API

```bash
# Get API endpoint
API_ENDPOINT=$(cd terraform && terraform output -raw api_endpoint)

# Test statistics endpoint
curl -X POST ${API_ENDPOINT}/statistics \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "test-session-001",
    "provider": "anthropic",
    "model": "claude-sonnet-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "waiting_for_human_ms": 30000,
    "ai_processing_ms": 2000,
    "tool_execution_ms": 1000,
    "tool_calls": {"Read": 3, "Write": 1},
    "tool_success_fail": {"Read": {"success": 3, "failed": 0}, "Write": {"success": 1, "failed": 0}}
  }'

# Test bugs endpoint
curl -X POST ${API_ENDPOINT}/bugs \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Example bug report for testing"
  }'
```

## Rust Client Library

A Rust client library is available at `client/` for easy integration:

```toml
[dependencies]
tycode-client = { git = "https://github.com/tigy32/TycodeCloud", branch = "main" }
```

**Usage:**

```rust
use tycode_client::{TycodeClient, SessionStatisticsBuilder};

let client = TycodeClient::new("https://your-api-endpoint.amazonaws.com");

let stats = SessionStatisticsBuilder::new("session-123", "anthropic", "claude-sonnet-4")
    .tokens(1500, 800)
    .timing(45000, 3200, 1800)
    .add_tool_call("Read", 5)
    .add_tool_stats("Read", 5, 0)
    .build();

client.report_statistics(&stats).await?;
```

See [`client/README.md`](client/README.md) for full documentation.

## Project Structure

```
TycodeCloud/
├── lambdas/
│   ├── statistics/     # Statistics recording Lambda
│   └── bugs/           # Bug reporting Lambda
├── shared/             # Shared library (DB models, utils)
├── client/             # Rust client library for API consumption
├── terraform/          # Infrastructure as Code
├── db/                 # Database schema and migrations
├── build.sh            # Lambda build script
└── README.md           # This file
```

## Development

### Building Locally

```bash
# Build all packages
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

### Updating Lambda Functions

1. Make code changes
2. Run `./build.sh` to rebuild
3. Run `cd terraform && terraform apply` to redeploy

## Cost Optimization

This setup is designed for minimal cost:

- **Lambda**: ARM64 architecture (20% cheaper than x86)
- **API Gateway**: HTTP API (up to 71% cheaper than REST API)
- **Aurora DSQL**: Serverless, pay-per-request pricing
- **Memory**: 256MB (minimum needed)
- **Timeout**: 30s (adjust if needed)

## Security Notes

- **Aurora DSQL Authentication**: Uses IAM-based authentication with auto-generated tokens (no passwords!)
- **Lambda Permissions**: IAM role allows `dsql:DbConnectAdmin` for token generation
- **API Access**: Publicly accessible (no authentication)
- **CORS**: Configured for cross-origin requests
- **Network**: Lambda functions run in VPC for database access
- **Transport**: All data transmitted over HTTPS

### How DSQL Authentication Works

The Lambda functions automatically generate short-lived authentication tokens using AWS IAM:

1. Lambda assumes IAM role with `dsql:DbConnectAdmin` permission
2. Rust code calls `AuthTokenGenerator.db_connect_admin_auth_token()`
3. Temporary token (valid ~15 minutes) is used to connect to DSQL
4. No passwords or long-lived credentials needed!

This is handled automatically by the `shared` library - see `shared/src/lib.rs` for implementation.

## Monitoring

View logs in CloudWatch:

```bash
# Statistics Lambda logs
aws logs tail /aws/lambda/tycode-api-statistics --follow

# Bugs Lambda logs
aws logs tail /aws/lambda/tycode-api-bugs --follow
```

## Cleanup

To remove all resources:

```bash
cd terraform
terraform destroy
```

## License

MIT

## Support

For issues or questions, please open an issue in the repository.

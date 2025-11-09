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

### 3. Configure Terraform Variables

Create `terraform/terraform.tfvars`:

```hcl
aws_region         = "us-west-2"
project_name       = "tycode-api"
db_admin_username  = "admin"
db_admin_password  = "YourSecurePassword123!"  # Change this!
```

**Important**: Never commit `terraform.tfvars` to version control!

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
    "tool_calls": {"Read": 3, "Write": 1}
  }'

# Test bugs endpoint
curl -X POST ${API_ENDPOINT}/bugs \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Example bug report for testing"
  }'
```

## Project Structure

```
TycodeCloud/
├── lambdas/
│   ├── statistics/     # Statistics recording Lambda
│   └── bugs/           # Bug reporting Lambda
├── shared/             # Shared library (DB models, utils)
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

- Database credentials are stored in Lambda environment variables
- API is publicly accessible (no authentication)
- CORS is configured for cross-origin requests
- Lambda functions run in VPC for database access
- All data transmitted over HTTPS

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

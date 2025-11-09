terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# Variables
variable "aws_region" {
  description = "AWS region for deployment"
  type        = string
  default     = "us-west-2"
}

variable "project_name" {
  description = "Project name for resource naming"
  type        = string
  default     = "tycode-api"
}

variable "db_admin_username" {
  description = "Database admin username"
  type        = string
  default     = "admin"
  sensitive   = true
}

variable "db_admin_password" {
  description = "Database admin password"
  type        = string
  sensitive   = true
}

# Data sources
data "aws_caller_identity" "current" {}

# Aurora DSQL Cluster
resource "aws_rds_cluster" "tycode_db" {
  cluster_identifier     = "${var.project_name}-cluster"
  engine                 = "aurora-dsql"
  master_username        = var.db_admin_username
  master_password        = var.db_admin_password
  database_name          = "tycode"
  skip_final_snapshot    = true

  tags = {
    Name        = "${var.project_name}-cluster"
    Environment = "production"
  }
}

# VPC for Lambda (if needed for DB access)
resource "aws_default_vpc" "default" {}

data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [aws_default_vpc.default.id]
  }
}

# Security Group for Lambda
resource "aws_security_group" "lambda_sg" {
  name        = "${var.project_name}-lambda-sg"
  description = "Security group for Lambda functions"
  vpc_id      = aws_default_vpc.default.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${var.project_name}-lambda-sg"
  }
}

# IAM Role for Lambda
resource "aws_iam_role" "lambda_role" {
  name = "${var.project_name}-lambda-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Name = "${var.project_name}-lambda-role"
  }
}

# Attach basic Lambda execution policy
resource "aws_iam_role_policy_attachment" "lambda_basic" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# Attach VPC execution policy
resource "aws_iam_role_policy_attachment" "lambda_vpc" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

# Lambda function for statistics
resource "aws_lambda_function" "statistics" {
  filename         = "${path.module}/../target/lambda/statistics/bootstrap.zip"
  function_name    = "${var.project_name}-statistics"
  role            = aws_iam_role.lambda_role.arn
  handler         = "bootstrap"
  source_code_hash = filebase64sha256("${path.module}/../target/lambda/statistics/bootstrap.zip")
  runtime         = "provided.al2023"
  architectures   = ["arm64"]
  timeout         = 30
  memory_size     = 256

  environment {
    variables = {
      DB_HOST     = aws_rds_cluster.tycode_db.endpoint
      DB_NAME     = "tycode"
      DB_USER     = var.db_admin_username
      DB_PASSWORD = var.db_admin_password
      RUST_LOG    = "info"
    }
  }

  vpc_config {
    subnet_ids         = data.aws_subnets.default.ids
    security_group_ids = [aws_security_group.lambda_sg.id]
  }

  tags = {
    Name = "${var.project_name}-statistics"
  }
}

# Lambda function for bugs
resource "aws_lambda_function" "bugs" {
  filename         = "${path.module}/../target/lambda/bugs/bootstrap.zip"
  function_name    = "${var.project_name}-bugs"
  role            = aws_iam_role.lambda_role.arn
  handler         = "bootstrap"
  source_code_hash = filebase64sha256("${path.module}/../target/lambda/bugs/bootstrap.zip")
  runtime         = "provided.al2023"
  architectures   = ["arm64"]
  timeout         = 30
  memory_size     = 256

  environment {
    variables = {
      DB_HOST     = aws_rds_cluster.tycode_db.endpoint
      DB_NAME     = "tycode"
      DB_USER     = var.db_admin_username
      DB_PASSWORD = var.db_admin_password
      RUST_LOG    = "info"
    }
  }

  vpc_config {
    subnet_ids         = data.aws_subnets.default.ids
    security_group_ids = [aws_security_group.lambda_sg.id]
  }

  tags = {
    Name = "${var.project_name}-bugs"
  }
}

# API Gateway HTTP API
resource "aws_apigatewayv2_api" "tycode_api" {
  name          = "${var.project_name}-http-api"
  protocol_type = "HTTP"
  description   = "Tycode statistics and bug reporting API"

  cors_configuration {
    allow_origins = ["*"]
    allow_methods = ["POST", "OPTIONS"]
    allow_headers = ["content-type"]
    max_age       = 300
  }

  tags = {
    Name = "${var.project_name}-api"
  }
}

# API Gateway Stage
resource "aws_apigatewayv2_stage" "default" {
  api_id      = aws_apigatewayv2_api.tycode_api.id
  name        = "$default"
  auto_deploy = true

  tags = {
    Name = "${var.project_name}-stage"
  }
}

# Lambda integration for statistics
resource "aws_apigatewayv2_integration" "statistics" {
  api_id             = aws_apigatewayv2_api.tycode_api.id
  integration_type   = "AWS_PROXY"
  integration_uri    = aws_lambda_function.statistics.invoke_arn
  integration_method = "POST"
  payload_format_version = "2.0"
}

# Lambda integration for bugs
resource "aws_apigatewayv2_integration" "bugs" {
  api_id             = aws_apigatewayv2_api.tycode_api.id
  integration_type   = "AWS_PROXY"
  integration_uri    = aws_lambda_function.bugs.invoke_arn
  integration_method = "POST"
  payload_format_version = "2.0"
}

# Route for statistics
resource "aws_apigatewayv2_route" "statistics" {
  api_id    = aws_apigatewayv2_api.tycode_api.id
  route_key = "POST /statistics"
  target    = "integrations/${aws_apigatewayv2_integration.statistics.id}"
}

# Route for bugs
resource "aws_apigatewayv2_route" "bugs" {
  api_id    = aws_apigatewayv2_api.tycode_api.id
  route_key = "POST /bugs"
  target    = "integrations/${aws_apigatewayv2_integration.bugs.id}"
}

# Lambda permissions for API Gateway
resource "aws_lambda_permission" "statistics_api" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.statistics.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.tycode_api.execution_arn}/*/*"
}

resource "aws_lambda_permission" "bugs_api" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.bugs.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.tycode_api.execution_arn}/*/*"
}

# Outputs
output "api_endpoint" {
  description = "API Gateway endpoint URL"
  value       = aws_apigatewayv2_api.tycode_api.api_endpoint
}

output "statistics_url" {
  description = "Statistics API endpoint"
  value       = "${aws_apigatewayv2_api.tycode_api.api_endpoint}/statistics"
}

output "bugs_url" {
  description = "Bugs API endpoint"
  value       = "${aws_apigatewayv2_api.tycode_api.api_endpoint}/bugs"
}

output "database_endpoint" {
  description = "Aurora DSQL cluster endpoint"
  value       = aws_rds_cluster.tycode_db.endpoint
  sensitive   = true
}

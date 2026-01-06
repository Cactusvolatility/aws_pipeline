terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "us-west-2"
}

# Budgeting

resource "aws_budgets_budget" "monthly_cost" {
  name         = "trading-system-monthly-budget"
  budget_type  = "COST"
  limit_amount = "50"
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  cost_filter {
    name = "Service"
    values = [
      "Amazon DynamoDB",
      "Amazon Simple Storage Service",
      "AWS Lambda",
      "Amazon CloudWatch"
    ]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 80
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = var.email
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 100
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = var.email
  }
}

resource "aws_budgets_budget" "dynamodb" {
  name         = "dynamodb-budget"
  budget_type  = "COST"
  limit_amount = "10"
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  cost_filter {
    name   = "Service"
    values = ["Amazon DynamoDB"]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 80
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = var.email
  }
}



# Scheduler

resource "aws_cloudwatch_event_rule" "tiingo_ingest" {
  name                = "minute-book-pull"
  description         = "Trigger tiingo minute top of the book pull"
  schedule_expression = var.enable_iex ? "cron(* 14-20 ? * MON-FRI *)" : "cron(0 0 31 2 ? *)"
}

resource "aws_cloudwatch_event_target" "tiingo_target" {
  rule      = aws_cloudwatch_event_rule.tiingo_ingest.name
  target_id = "Tiingo-Ingest"
  arn       = aws_lambda_function.tiingo_iex.arn
}

resource "aws_cloudwatch_event_rule" "fmp_fetch" {
  name                = "fmp-news-fetch"
  description         = "Trigger news collection every hour"
  schedule_expression = var.enable_fmp == true ? "cron(0 * * * ? *)" : "cron(0 0 31 2 ? *)"
}

output "iex_enabled" {
  value = var.enable_iex
}

resource "aws_cloudwatch_event_target" "fmp_target" {
  rule      = aws_cloudwatch_event_rule.fmp_fetch.name
  target_id = "FMP-Fetch"
  arn       = aws_lambda_function.fmp_news.arn
}


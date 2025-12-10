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


# DynamoDB_state

resource "aws_dynamodb_table" "ticker_state" {
  name         = "ticker-ingestion-state"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "ticker"

  attribute {
    name = "ticker"
    type = "S"
  }

  tags = {
    Environment = "dev"
  }
}

# IAM

resource "aws_iam_role_policy" "lambda_dlq_access" {
  role = aws_iam_role.lambda_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["sqs:SendMessage"]
        Resource = aws_sqs_queue.fn_dlq.arn
      }
    ]
  })
}

data "aws_iam_policy_document" "lambda_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "lambda_role" {
  name_prefix        = "stock-fetcher-lambda-"
  assume_role_policy = data.aws_iam_policy_document.lambda_trust.json
}

data "aws_iam_policy_document" "dynamodb_access" {
  statement {
    effect = "Allow"

    actions = [
      "dynamodb:GetItem",
      "dynamodb:PutItem",
      "dynamodb:UpdateItem",
      "dynamodb:BatchGetItem",
      "dynamodb:BatchWriteItem",
      "dynamodb:Query",
      "dynamodb:Scan",
      "dynamodb:DescribeTable"
    ]

    resources = [
      aws_dynamodb_table.ticker_state.arn,
      "${aws_dynamodb_table.ticker_state.arn}/index/*"
    ]
  }
}

resource "aws_iam_role_policy" "dynamodb_access" {
  name   = "dynamodb-access"
  role   = aws_iam_role.lambda_role.id
  policy = data.aws_iam_policy_document.dynamodb_access.json
}

resource "aws_iam_role_policy_attachment" "lambda_basic" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

data "aws_iam_policy_document" "s3_access" {
  statement {
    effect = "Allow"
    actions = [
      "s3:PutObject",
      "s3:PutObjectAcl",
    ]
    resources = [
      "${aws_s3_bucket.data_lake.arn}/*"
    ]
  }
}

resource "aws_iam_role_policy" "s3_access" {
  name   = "s3-access"
  role   = aws_iam_role.lambda_role.id
  policy = data.aws_iam_policy_document.s3_access.json
}

# Lambda

resource "aws_lambda_function" "fetcher" {
  filename      = "../dist/fetcher/fetcher.zip"
  function_name = "stock-fetcher"
  role          = aws_iam_role.lambda_role.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]

  source_code_hash = filebase64sha256("../dist/fetcher/fetcher.zip")

  environment {
    variables = {
      DYNAMODB_TABLE = aws_dynamodb_table.ticker_state.name
      S3_BUCKET = aws_s3_bucket.data_lake.id
      TICKERS        = "TSM,INTC,UMC,GFS,MU,AMD"
      BUILD_TIME     = timestamp()
      TIINGO_API_KEY = var.tiingo_api_key
      FMP_API_KEY = var.fmp_api_key
    }
  }

  dead_letter_config {
    target_arn = aws_sqs_queue.fn_dlq.arn
  }
}

resource "aws_sqs_queue" "fn_dlq" {
  name = "fetcher-dlq"
}

resource "aws_sqs_queue_policy" "fn_dlq_policy" {
  queue_url = aws_sqs_queue.fn_dlq.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action    = "sqs:SendMessage"
      Resource  = aws_sqs_queue.fn_dlq.arn
      Condition = {
        ArnEquals = { "AWS:SourceArn" = aws_lambda_function.fetcher.arn }
      }
    }]
  })
}

# S3

resource "aws_s3_bucket" "terraform_state" {
  bucket        = "tf-state-data-pipeline-456xyz"
  force_destroy = false

  tags = {
    Name        = "Terraform State"
    Environment = "dev"
  }
}

resource "aws_s3_bucket_versioning" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket" "data_lake" {
  bucket = "semidata-lake-123456"

  tags = {
    Name        = "Stock Data Lake"
    Environment = "dev"
  }
}

resource "aws_s3_bucket_versioning" "data_lake" {
  bucket = aws_s3_bucket.data_lake.id
  versioning_configuration {
    status = "Enabled"
  }
}

# Scheduler

resource "aws_cloudwatch_event_rule" "daily_fetch" {
  name                = "daily-stock-fetch"
  description         = "Trigger stock data fetch on weekdays"
  schedule_expression = var.enable_ingestion == true ? "cron(* * * * ? *)" : "cron(0 0 31 2 ? *)"
}

resource "aws_cloudwatch_event_target" "lambda_target" {
  rule      = aws_cloudwatch_event_rule.daily_fetch.name
  target_id = "StockDataFetcherTarget"
  arn       = aws_lambda_function.fetcher.arn
}


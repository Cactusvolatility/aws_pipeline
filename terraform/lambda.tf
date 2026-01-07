
# Lambda

resource "aws_lambda_function" "tiingo_iex" {
  filename      = "../dist/tiingo_iex/tiingo_iex.zip"
  function_name = "tiingo-iex-ingest"
  role          = aws_iam_role.lambda_role.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]

  source_code_hash = filebase64sha256("../dist/tiingo_iex/tiingo_iex.zip")

  environment {
    variables = {
      S3_BUCKET      = aws_s3_bucket.data_lake.id
      TICKERS        = join(",", var.tickers)
      BUILD_TIME     = timestamp()
      TIINGO_API_KEY = var.tiingo_api_key
      FMP_API_KEY    = var.fmp_api_key
    }
  }

  dead_letter_config {
    target_arn = aws_sqs_queue.fn_dlq.arn
  }
}

resource "aws_lambda_function" "fmp_news" {
  filename      = "../dist/fmp_news/fmp_news.zip"
  function_name = "fmp-news-ingest"
  role          = aws_iam_role.lambda_role.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]

  source_code_hash = filebase64sha256("../dist/fmp_news/fmp_news.zip")

  environment {
    variables = {
      DYNAMODB_TABLE = aws_dynamodb_table.news_articles.name
      S3_BUCKET      = aws_s3_bucket.data_lake.id
      TICKERS        = join(",", var.tickers)
      BUILD_TIME     = timestamp()
      TIINGO_API_KEY = var.tiingo_api_key
      FMP_API_KEY    = var.fmp_api_key
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
        ArnEquals = { "AWS:SourceArn" = [
          aws_lambda_function.tiingo_iex.arn,
          aws_lambda_function.fmp_news.arn
          ]
        }
      }
    }]
  })
}

# Python

resource "aws_lambda_function" "py_process_5min" {
  function_name = "py-process-5min"
  role          = aws_iam_role.py_process_5min_role.arn

  package_type     = "Image"
  image_uri        = "${aws_ecr_repository.process_5min.repository_url}:latest"
  source_code_hash = timestamp()

  architectures = ["arm64"]

  timeout     = 60
  memory_size = 1024

  environment {
    variables = {
      S3_BUCKET      = aws_s3_bucket.data_lake.bucket
      DYNAMODB_TABLE = aws_dynamodb_table.ticker_state.name
      PREFIX_BASE    = "prices/tiingo_book"
    }
  }
}

# ECR

resource "aws_ecr_repository" "process_5min" {
  name = "process-5min-lambda"
}

resource "aws_ecr_lifecycle_policy" "process_5min" {
  repository = aws_ecr_repository.process_5min.name
  policy = jsonencode({
    rules = [{
      rulePriority = 1
      description  = "Keep last 20 images"
      selection = {
        tagStatus   = "any"
        countType   = "imageCountMoreThan"
        countNumber = 20
      }
      action = { type = "expire" }
    }]
  })
}
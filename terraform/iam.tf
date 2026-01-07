
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
      "${aws_dynamodb_table.ticker_state.arn}/index/*",
      aws_dynamodb_table.news_articles.arn,
      "${aws_dynamodb_table.news_articles.arn}/index/*"
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

resource "aws_lambda_permission" "allow_eventbridge_tiingo" {
  statement_id  = "AllowExecutionFromEventBridgeTiingo"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.tiingo_iex.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.tiingo_ingest.arn
}

resource "aws_lambda_permission" "allow_eventbridge_fmp" {
  statement_id  = "AllowExecutionFromEventBridgeFmp"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.fmp_news.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.fmp_fetch.arn
}

resource "aws_lambda_permission" "allow_eventbridge_py_process_5min" {
  statement_id  = "AllowExecutionFromEventBridgePyProcess5min"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.py_process_5min.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.py_process_5min_schedule.arn
}

# Python lambdas---

data "aws_iam_policy_document" "py_lambda_trust" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "py_process_5min_role" {
  name_prefix        = "py-process-5min-"
  assume_role_policy = data.aws_iam_policy_document.py_lambda_trust.json
}

resource "aws_iam_role_policy_attachment" "py_process_5min_basic" {
  role       = aws_iam_role.py_process_5min_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# S3 read
data "aws_iam_policy_document" "py_process_5min_s3" {
  statement {
    effect    = "Allow"
    actions   = ["s3:ListBucket"]
    resources = [aws_s3_bucket.data_lake.arn]
    condition {
      test     = "StringLike"
      variable = "s3:prefix"
      values   = ["prices/tiingo_book/*"]
    }
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:GetObject"]
    resources = ["${aws_s3_bucket.data_lake.arn}/prices/tiingo_book/*"]
  }
}

resource "aws_iam_role_policy" "py_process_5min_s3" {
  name   = "py-process-5min-s3"
  role   = aws_iam_role.py_process_5min_role.id
  policy = data.aws_iam_policy_document.py_process_5min_s3.json
}

# DynamoDB access

data "aws_iam_policy_document" "py_process_5min_dynamo" {
  statement {
    effect = "Allow"
    actions = [
      "dynamodb:GetItem",
      "dynamodb:PutItem",
      "dynamodb:UpdateItem",
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

resource "aws_iam_role_policy" "py_process_5min_dynamo" {
  name   = "py-process-5min-dynamo"
  role   = aws_iam_role.py_process_5min_role.id
  policy = data.aws_iam_policy_document.py_process_5min_dynamo.json
}
# DynamoDB_state

resource "aws_dynamodb_table" "ticker_state" {
  name         = "ticker-statistics"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "ticker"
  range_key    = "window_timestamp"

  attribute {
    name = "ticker"
    type = "S"
  }

  attribute {
    name = "window_timestamp"
    type = "N"
  }

  ttl {
    attribute_name = "ttl"
    enabled        = true
  }

  tags = {
    Environment = "dev"
  }
}

resource "aws_dynamodb_table" "news_articles" {
  name         = "news-articles-tracking"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "article_id"

  attribute {
    name = "article_id"
    type = "S"
  }

  attribute {
    name = "ticker"
    type = "S"
  }

  attribute {
    name = "published_at"
    type = "N"
  }

  # GSI for querying by ticker
  global_secondary_index {
    name            = "ticker-published-index"
    hash_key        = "ticker"
    range_key       = "published_at"
    projection_type = "KEYS_ONLY"
  }

  ttl {
    attribute_name = "ttl"
    enabled        = true
  }

  tags = {
    Environment = "dev"
    Purpose     = "article-deduplication"
  }
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
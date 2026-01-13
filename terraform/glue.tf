# S3

resource "aws_s3_object" "eod_script" {
  bucket = aws_s3_bucket.artifacts.id
  key    = "glue/eod_job.py"
  source = "${path.module}/../glue/eod_job.py"
  etag   = filemd5("${path.module}/../glue/eod_job.py")
}

resource "aws_glue_job" "eod_job" {
  name     = "stock-eod-processing"
  role_arn = aws_iam_role.glue_role.arn

  command {
    name            = "pythonshell"
    script_location = "s3://${aws_s3_bucket.artifacts.bucket}/${aws_s3_object.eod_script.key}"
    python_version  = "3.9"
  }

  default_arguments = {
    "--job-language" = "python"
    "--bucket"       = "semidata-lake-123456"
  }

  max_retries  = 0
  timeout      = 30
  glue_version = "4.0"
}

# schedule glue trigger

resource "aws_glue_trigger" "eod_daily" {
  name     = "eod-daily"
  type     = "SCHEDULED"
  schedule = "cron(5 20 * * ? *)"

  actions {
    job_name = aws_glue_job.eod_job.name
  }
}

# IAM

# Glue service role
resource "aws_iam_role" "glue_role" {
  name = "glue-eod-job-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "glue.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "glue_service" {
  role       = aws_iam_role.glue_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSGlueServiceRole"
}

data "aws_iam_policy_document" "glue_s3" {
  statement {
    effect = "Allow"
    actions = [
      "s3:ListBucket",
      "s3:GetBucketLocation"
    ]
    resources = [
      aws_s3_bucket.data_lake.arn,
      aws_s3_bucket.artifacts.arn
    ]
  }

  statement {
    effect = "Allow"
    actions = [
      "s3:GetObject",
      "s3:PutObject",
      "s3:DeleteObject"
    ]
    resources = [
      "${aws_s3_bucket.data_lake.arn}/*",
      "${aws_s3_bucket.artifacts.arn}/glue/*"
    ]
  }
}

resource "aws_iam_role_policy" "glue_s3" {
  name   = "glue-eod-s3-access"
  role   = aws_iam_role.glue_role.id
  policy = data.aws_iam_policy_document.glue_s3.json
}
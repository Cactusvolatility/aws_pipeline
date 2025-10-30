## README

This is a data ingestion pipeline using a Lambda to pull data from Tiingo to write to DynamoDB and then doing some small analysis. This is a project to gain knowledge of the AWS tooling and some platform experience. Originally Yahoo Finance's API was going to be used for bulk pulls but after coming across inconsistent pulls and call limits we switch to Tiingo.

## Overview
- AWS Lambda (Rust) Runtime: Fetches daily data with optional backfill capability
- DynamoDB: Store price and volume data
- Eventbridge: Triggers based on rules/events
- Terraform (IaC): Handles AWS infrastructure
- Cloudwatch: Metrics, logs, alerts
- Python: Simple pull to graph some of the charts
- TODO: Add additoinal graphs to some API endpoint

## Features
- Backfill mode via Lambda
- Automatic retry and throttling control
- Least-privilege IAM roles
- Fully managed, serverless design

## Deployment
terraform init
terraform apply
(note: will need to input Tiingo credentials)

## Invocation
When running locally, credentials are loaded from the default AWS CLI profile or environment.

Regular daily pull:
```bash
aws lambda invoke --function-name stock-fetcher --payload '{}' response.json
```

Example invoke is below for backfill (need to specify output due to AWS CLi v2):

```bash
aws lambda invoke \
  --function-name stock-fetcher \
  --payload '{"mode":"backfill","start":"2025-09-01","end":"2025-10-01","tickers":"AAPL,MSFT"}' \
  --cli-binary-format raw-in-base64-out \
  response.json
```

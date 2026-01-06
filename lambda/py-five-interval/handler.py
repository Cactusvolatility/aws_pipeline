import polars as pl
import boto3
from datetime import datetime, timezone

s3_client = boto3.client('s3')
dynamodb = boto3.resource('dynamodb')

BUCKET = 'semidata-lake-123456'
State_table = 'ticker-statistics'

def trigger_five_handler(event, context):
    print("trigger lambda for processing data")

    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")

    prefix = f'prices/tiingo_book/date=2026-01-05/'

    resp = s3_client.list_objects_v2(
        Bucket = BUCKET,
        Prefix = prefix,
    )

    if "Contents" not in resp:
        print("No files found")
        return {"files": []}

    keys = [obj["Key"] for obj in resp["Contents"]]

    print("Found files:")
    for k in keys:
        print(k)

    return {
        "date": today,
        "file_count": len(keys),
        "files": keys,
    }
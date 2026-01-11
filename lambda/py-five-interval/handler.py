import polars as pl
import boto3
from datetime import datetime, timedelta
from zoneinfo import ZoneInfo 

from analysis import parse_dt_from_key, calc_metrics
from aws_io import write_aws

s3_client = boto3.client('s3')
dynamodb = boto3.resource('dynamodb')

BUCKET = 'semidata-lake-123456'
State_table = 'ticker-statistics'

def trigger_five_handler(event, context):
    print("trigger lambda for processing data")

    # calculate timezones
    NY = ZoneInfo("America/New_York")
    now = datetime.now(tz=NY)
    effective = now - timedelta(minutes=1)
    start = effective.replace(minute=(effective.minute // 5) * 5, second=0, microsecond=0)
    end = start + timedelta(minutes=5)
    prefix = f"prices/tiingo_book/raw/date={start:%Y-%m-%d}/hour={start:%H}/" 

    resp = s3_client.list_objects_v2(
        Bucket = BUCKET,
        Prefix = prefix,
    )

    keys = [o["Key"] for o in resp.get("Contents", [])]

    window_keys = []
    for k in keys:
        dt = parse_dt_from_key(k).replace(tzinfo=NY)
        # we make sure it's between the start + end
        if start <= dt < end:
            window_keys.append((k,dt))
    
    if not window_keys:
        return {
            "statusCode": 200,
            "body": "no valid files in window"
        }
    
    bucket_keys = [k for k,_ in window_keys]

    s3_uris = [f"s3://{BUCKET}/{k}" for k in bucket_keys]
    df = pl.read_parquet(s3_uris)

    results = []

    for ticker, g in df.group_by("ticker", maintain_order=True):
        metrics = calc_metrics(g.sort("timestamp"))
        metrics["ticker"] = ticker
        metrics["analysis_ts"] = now
        results.append(metrics)

    features_df = pl.DataFrame(results)

    write_aws(features_df, window_start=start)


    return {
        'statusCode': 200,
        'body': f'Processed {len(bucket_keys)} files, {len(df)} rows'
    }
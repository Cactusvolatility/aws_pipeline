import polars as pl
import boto3
import math
from datetime import datetime, timedelta
from zoneinfo import ZoneInfo 

s3_client = boto3.client('s3')
dynamodb = boto3.resource('dynamodb')
BUCKET = 'semidata-lake-123456'
table = dynamodb.Table("ticker-statistics")

def write_aws(features_df, window_start):
    for row in features_df.iter_rows(named=True):
        item = {
            "ticker": row["ticker"],
            "window_timestamp": int(window_start.timestamp()),
            "analysis_ts": row["analysis_ts"].isoformat(),
            "ttl": int((row["analysis_ts"] + timedelta(days=7)).timestamp()),
        }

        for k, v in row.items():
            if k in ("ticker", "analysis_ts"):
                continue
            if v is None:
                continue
            fv = float(v)
            if not math.isfinite(fv):
                continue
            item[k] = fv

        table.put_item(Item=item)

    features_df.write_parquet(
        f"s3://{BUCKET}/prices/tiingo_book/features/"
        f"interval=5m/date={window_start:%Y-%m-%d}/hour={window_start:%H}/"
        f"features_{window_start:%Y%m%d_%H%M%S}.parquet"
    )
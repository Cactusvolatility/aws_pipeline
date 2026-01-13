import polars as pl
import math
from datetime import datetime

def parse_dt_from_key(key):
    """Extract timestamp from parquet"""
    name = key.split('/')[-1].replace('.parquet', '')
    parts = name.split('_')
    date_str = parts[1]  # YYYYMMDD
    time_str = parts[2]  # HHMMSS
    return datetime.strptime(f"{date_str}{time_str}", "%Y%m%d%H%M%S")

def _safe_float(x):
    if x is None:
        return None
    try:
        x = float(x)
    except Exception:
        return None
    return x if math.isfinite(x) else None

def calc_metrics(df):
    # MID for state/volatility

    # do I need to srt?
    df = df.sort("timestamp")
    
    ohlc = df.select(
        pl.col("open").first(),
        pl.col("high").max(),
        pl.col("low").min(),
        pl.col("tngoLast").last().alias("close"),
    )

    open_ = _safe_float(ohlc["open"][0])
    high_ = _safe_float(ohlc["high"][0])
    low_ = _safe_float(ohlc["low"][0])
    close_ = _safe_float(ohlc["close"][0])

    if any(v is None for v in [open_, high_, low_, close_]):
        return None
    if open_ <= 0 or high_ <= 0 or low_ <= 0:
        return None
    if high_ < low_:
        return None
    
    # Calculate metrics with safe_float wrapping
    range_pct = _safe_float((high_ - low_) / open_)

    # mid log returns (intrawindow)
    df = df.with_columns(
        pl.col("mid").log().diff().alias("mid_log_ret")
    )
    realized_vol = _safe_float(df.select(pl.col("mid_log_ret").std()).item())

    # Last provides execution pressure
    df = df.with_columns(
        (pl.col("last") - pl.col("mid")).alias("last_mid_diff")
    )
    mean_last_mid = _safe_float(df.select(pl.col("last_mid_diff").mean()).item())
    max_last_mid = _safe_float(df.select(pl.col("last_mid_diff").abs().max()).item())
    

    return {
        "open": open_,
        "high": high_,
        "low": low_,
        "close": close_,
        #"log_return": log_return,
        "realized_vol": realized_vol,
        "range_pct": range_pct,
        "mean_last_mid": mean_last_mid,
        "max_last_mid": max_last_mid
    }
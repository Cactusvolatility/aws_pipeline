import polars as pl
from datetime import datetime

def parse_dt_from_key(key):
    """Extract timestamp from parquet"""
    name = key.split('/')[-1].replace('.parquet', '')
    parts = name.split('_')
    date_str = parts[1]  # YYYYMMDD
    time_str = parts[2]  # HHMMSS
    return datetime.strptime(f"{date_str}{time_str}", "%Y%m%d%H%M%S")

def calc_metrics(df):
    # MID for state/volatility
    ohlc = df.select(
        pl.col("mid").first().alias("open"),
        pl.col("mid").max().alias("high"),
        pl.col("mid").min().alias("low"),
        pl.col("mid").last().alias("close"),
    )

    open_ = ohlc["open"][0]
    high_ = ohlc["high"][0]
    low_  = ohlc["low"][0]
    close_ = ohlc["close"][0]

    # mid log returns (intrawindow)
    df = df.with_columns(
        pl.col("mid").log().diff().alias("mid_log_ret")
    )

    realized_vol = df.select(pl.col("mid_log_ret").std()).item()

    range_pct = (high_ - low_) / open_

    # Last provides execution pressure
    df = df.with_columns(
        (pl.col("last") - pl.col("mid")).alias("last_mid_diff")
    )

    mean_last_mid = df.select(pl.col("last_mid_diff").mean()).item()
    max_last_mid = df.select(pl.col("last_mid_diff").abs().max()).item()

    # bid ask is friction
    df = df.with_columns(
        (pl.col("ask") - pl.col("bid")).alias("spread")
    )

    avg_spread = df.select(pl.col("spread").mean()).item()
    max_spread = df.select(pl.col("spread").max()).item()

    return {
        "open": open_,
        "high": high_,
        "low": low_,
        "close": close_,
        #"log_return": log_return,
        "realized_vol": realized_vol,
        "range_pct": range_pct,
        "mean_last_mid": mean_last_mid,
        "max_last_mid": max_last_mid,
        "avg_spread": avg_spread,
        "max_spread": max_spread,
    }
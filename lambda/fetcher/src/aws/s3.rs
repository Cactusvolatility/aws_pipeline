use crate::models::{TickerBar, TiingoBook};
use anyhow::{Ok, Result};
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use chrono::Utc;
use std::sync::Arc;

use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;


// See Arrow docmentation
// https://docs.rs/arrow/latest/arrow/#tabular-representation


// this is now the backfill write
pub async fn write_tiingo_s3(
    client: &S3Client,
    bucket: &str,
    bars: &[TickerBar],
) -> Result<()> {
    // base case - if we get nothing from lambda
    if bars.is_empty() {
        return Ok(());
    }

    // grab the first 10 digits of the 0-index value
        // TODO: check if edge case like over midnight or somethign ruins this (concurrent pull fails)
    let first_bar_date = &bars[0].date[..10];
    let ts = Utc::now().format("%Y-%m-%d");

    let key = format!(
        "prices/tiingo/date={}/bars_{}.parquet",
        first_bar_date,
        ts,
    );
    println!("Write_s3: set up s3");

    let parquet_bytes = serialize_to_parquet(bars)?;

    println!("Write_s3: made parquet");

    // Bytestream wrapper supports streaming
    client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .body(ByteStream::from(parquet_bytes))
        .content_type("application/octet-stream")
        .send()
        .await?;

    println!("end s3 write, {} bars to s3://{}/{}", bars.len(), bucket, key);

    Ok(())
}

fn serialize_to_parquet(bars: &[TickerBar]) -> Result<Vec<u8>> {
    let schema = Schema::new(vec![
        Field::new("ticker", DataType::Utf8, false),
        Field::new("date", DataType::Utf8, false),
        Field::new("open", DataType::Float64, false),
        Field::new("high", DataType::Float64, false),
        Field::new("low", DataType::Float64, false),
        Field::new("close", DataType::Float64, false),
        Field::new("volume", DataType::Float64, false),
    ]);

    let tickers: StringArray = bars
        .iter()
        .map(|b| Some(b.ticker.as_str()))
        .collect();

    let dates: StringArray = bars
        .iter()
        .map(|b| Some(b.date.as_str()))
        .collect();

    let opens: Float64Array = bars
        .iter()
        .map(|b| Some(b.open))
        .collect();

    let highs: Float64Array = bars
        .iter()
        .map(|b| Some(b.high))
        .collect();

    let lows: Float64Array = bars
        .iter()
        .map(|b| Some(b.low))
        .collect();

    let closes: Float64Array = bars
        .iter()
        .map(|b| Some(b.close))
        .collect();

    let volumes: Float64Array = bars
        .iter()
        .map(|b| Some(b.volume))
        .collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(tickers),
            Arc::new(dates),
            Arc::new(opens),
            Arc::new(highs),
            Arc::new(lows),
            Arc::new(closes),
            Arc::new(volumes),
        ],
    )?;

    println!("serialize: finished setting up schema");

    let mut buffer = Vec::new();
    // TODO: 
    // if input file starts growing too large we need to add compression
    // 12/5: 100 tickers/min isn't large enough 
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(&mut buffer, batch.schema(), Some(props))?;
    writer.write(&batch)?;
    writer.close()?;

    Ok(buffer)


}

pub async fn write_book_s3(
    client: &S3Client,
    bucket: &str,
    books: &[TiingoBook],
) -> Result<()> {

    if books.is_empty() {
        // let caller handle error case
        return Ok(());
    }

    let datetime = Utc::now();
    let date = datetime.format("%Y-%m-%d");
    let file_ts = datetime.format("%Y%m%d_%H%M%S");

    let key = format!(
        "prices/tiingo_book/date={}/raw/book_{}.parquet",
        date, file_ts
    );

    let parquet_bytes = serialize_books_parquet(books)?;
        client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .body(ByteStream::from(parquet_bytes))
        .content_type("application/octet-stream")
        .send()
        .await?;

    Ok(())
}

fn serialize_books_parquet(books: &[TiingoBook]) -> Result<Vec<u8>> {
    let schema = Schema::new(vec![
        Field::new("ticker", DataType::Utf8, false),
        Field::new("timestamp", DataType::Utf8, false),
        Field::new("lastSaleTimestamp", DataType::Utf8, true),
        Field::new("quoteTimestamp", DataType::Utf8, true),
        Field::new("open", DataType::Float64, false),
        Field::new("high", DataType::Float64, false),
        Field::new("low", DataType::Float64, false),
        Field::new("mid", DataType::Float64, true),
        Field::new("tngoLast", DataType::Float64, false),
        Field::new("last", DataType::Float64, true),
        Field::new("lastSize", DataType::Int64, true),
        Field::new("bidSize", DataType::Int64, true),
        Field::new("bidPrice", DataType::Float64, true),
        Field::new("askPrice", DataType::Float64, true),
        Field::new("askSize", DataType::Int64, true),
        Field::new("volume", DataType::Int64, false),
        Field::new("prevClose", DataType::Float64, false),
    ]);
    
    let ticker: StringArray = books
        .iter()
        .map(|b| Some(b.ticker.as_str()))
        .collect();

    let timestamp: StringArray = books
        .iter()
        .map(|b| Some(b.timestamp.as_str()))
        .collect();

    let last_sale_ts: StringArray = books
        .iter()
        .map(|b| b.lastSaleTimestamp.as_deref())
        .collect();

    let quote_ts: StringArray = books
        .iter()
        .map(|b| b.quoteTimestamp.as_deref())
        .collect();

    let open: Float64Array = books
        .iter()
        .map(|b| Some(b.open))
        .collect();

    let high: Float64Array = books
        .iter()
        .map(|b| Some(b.high))
        .collect();

    let low: Float64Array = books
        .iter()
        .map(|b| Some(b.low))
        .collect();

    let mid: Float64Array = books
        .iter()
        .map(|b| b.mid)
        .collect();

    let tngo_last: Float64Array = books
        .iter()
        .map(|b| Some(b.tngoLast))
        .collect();

    let last: Float64Array = books
        .iter()
        .map(|b| b.last)
        .collect();

    let last_size: Int64Array = books
        .iter()
        .map(|b| b.lastSize)
        .collect();

    let bid_size: Int64Array = books
        .iter()
        .map(|b| b.bidSize)
        .collect();

    let bid_price: Float64Array = books
        .iter()
        .map(|b| b.bidPrice)
        .collect();

    let ask_price: Float64Array = books
        .iter()
        .map(|b| b.askPrice)
        .collect();

    let ask_size: Int64Array = books
        .iter()
        .map(|b| b.askSize)
        .collect();

    let volume: Int64Array = books
        .iter()
        .map(|b| Some(b.volume))
        .collect();

    let prev_close: Float64Array = books
        .iter()
        .map(|b| Some(b.prevClose))
        .collect();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(ticker),
            Arc::new(timestamp),
            Arc::new(last_sale_ts),
            Arc::new(quote_ts),
            Arc::new(open),
            Arc::new(high),
            Arc::new(low),
            Arc::new(mid),
            Arc::new(tngo_last),
            Arc::new(last),
            Arc::new(last_size),
            Arc::new(bid_size),
            Arc::new(bid_price),
            Arc::new(ask_price),
            Arc::new(ask_size),
            Arc::new(volume),
            Arc::new(prev_close),
        ],
    )?;

    println!("serialize: finished setting up schema for book");

    let mut buffer = Vec::new();
    // TODO: 
    // if input file starts growing too large we need to add compression
    // 12/5: 100 tickers/min isn't large enough 
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(&mut buffer, batch.schema(), Some(props))?;
    writer.write(&batch)?;
    writer.close()?;

    Ok(buffer)

}
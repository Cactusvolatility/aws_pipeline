use anyhow::{Result, Context};
use reqwest::Client;
use crate::models::TiingoBar;

/* 
    Since we're no longer writing to DynamoDB directly and instead downloading to S3 we simplify the process

    12/5/25:
        1. change to 1 min microbatch
        2. change backfill to just pull from date to curr
*/

/*
    Additional inputs can be found at:
    https://www.tiingo.com/documentation/iex
*/
pub async fn fetch_backfill (
    client: &Client,
    ticker: &str,
    start_date: &str,
    end_date: Option<&str>,
    api_key: &str,
) -> Result<Vec<TiingoBar>> {
    //let test_date = "2025-12-08";
    let mut url = format!(
        "https://api.tiingo.com/iex/{ticker}/prices?token={api_key}&startDate={start_date}&resampleFreq=1min&columns=date,close,high,low,open,volume"
    );

    if let Some(end) = end_date {
        url.push_str(&format!("&endDate={end}"));
    }

    let resp = client.get(&url).send().await
        .context(format!("Failed to fetch data for {}", ticker))?;

    let resp = resp.error_for_status()
        .context(format!("API returned error for {}", ticker))?;

    println!("fetch_intraday: received json");
    
    let bars: Vec<TiingoBar> = resp.json().await?;
    println!("fetch_intraday: parsed to TiingoBar Struct");
    Ok(bars)
}


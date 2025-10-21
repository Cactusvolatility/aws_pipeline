use serde::{Deserialize, Serialize};
use anyhow::Result;
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;

#[derive(Deserialize)]
pub struct Request {
    pub mode: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub tickers: Option<String>,
}

#[derive(Serialize)]
pub struct Response {
    pub message: String,
    pub fetched_count: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TiingoJSON {
    pub date: String,
    pub open: f64,
    pub close: f64,
    pub volume: i64,
}

#[derive(Debug, Clone)]
pub struct Quote {
    pub ticker: String,
    pub date: String,
    pub close: f64,
    pub volume: i64,
}

// I should be batching this and the DynamoDB writes
async fn fetch_chunk(
    client: Client, 
    chunk: Vec<String>,
    api_key: &str,
) -> Result<Vec<Quote>> {

    let mut all = Vec::new();

    for ticker in chunk {
        let url = format!(
            "https://api.tiingo.com/tiingo/daily/{}/prices?token={}",
            ticker, api_key
        );

        let resp = client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;

        let data: Vec<TiingoJSON> = resp.json().await?;
        if let Some(pulled_data) = data.last() {
            all.push(Quote {
                ticker: ticker.clone(),
                date: pulled_data.date.clone(),
                close: pulled_data.close,
                volume: pulled_data.volume,
            });
        }
    }

    Ok(all)

    /*
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 ... Safari/537.36")
        .header("Accept", "application)
        .header("accept-language", "en-US,en;q=0.9")
        .send()
        .await?
        .error_for_status()?;

    let y: Yresponse = resp.json().await?;
    Ok(y.quote_response.result)
    */
}

pub async fn backfill_quotes(
    payload: Request,
    api_key: &str,
) -> Result<Vec<Quote>> {
    let start = payload.start.as_deref().ok_or_else(|| anyhow::anyhow!("start required"))?;
    let end   = payload.end.as_deref().ok_or_else(|| anyhow::anyhow!("end required"))?;

    let tickers_str = payload.tickers.as_deref().unwrap_or("AAPL");
    let mut tickers: Vec<String> = tickers_str.split(',').map(|s| s.trim().to_string()).collect();
    tickers.sort();
    tickers.dedup();

    let client = reqwest::Client::new();
    let mut quotes = Vec::new();

    for t in tickers {
        println!("Backfilling {} from {} to {}", t, start, end);
        let url = format!(
            "https://api.tiingo.com/tiingo/daily/{}/prices?startDate={}&endDate={}&token={}",
            t, start, end, api_key
        );
        let resp = client.get(&url).send().await?.error_for_status()?;
        let days: Vec<TiingoJSON> = resp.json().await?;

        for d in days {
            quotes.push(Quote {
                ticker: t.clone(),
                date:   d.date,
                close:  d.close,
                volume: d.volume,
            });
        }
    }
    Ok(quotes)
}

pub async fn fetch_batch_quotes(
    tickers: &[String],
    chunk_size: usize,
    max_concurrency: usize,
    api_key: &str,
) -> Result<Vec<Quote>> {
    let client = reqwest::Client::new();

    let mut unique = tickers.iter().filter(|ticker| !ticker.is_empty()).cloned().collect::<Vec<_>>();
    unique.sort();
    unique.dedup();

    let chunks: Vec<Vec<String>> = unique.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();

    // ????? what the heck
        // error
    let mut in_flight = FuturesUnordered::new();
    let mut quotes = Vec::new();
    let mut next_idx = 0;

    // limit calls
    while next_idx < chunks.len() && in_flight.len() < max_concurrency {
        // just clone it
            // give it to the future
        let chunk = chunks[next_idx].clone();
        in_flight.push(fetch_chunk(client.clone(), chunk, api_key));
        next_idx += 1;
    }

    while let Some(result) = in_flight.next().await {
        let mut batch = result?;
        quotes.append(&mut batch);

        if next_idx < chunks.len() {
            let chunk = chunks[next_idx].clone();
            in_flight.push(fetch_chunk(client.clone(), chunk, api_key));
            next_idx += 1;
        }
    }

    println!("Finish concurrent pull");

    Ok(quotes)
}
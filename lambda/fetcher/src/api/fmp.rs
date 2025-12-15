use anyhow::{Result, Context};
use reqwest::Client;
use crate::models::{FmpNews};

pub async fn fetch_fmp_news(
    client: &Client,
    tickers: &str,
    start_date: &str,
    api_key: &str,
) -> Result<Vec<FmpNews>> {
    
    /*
        since we're only pulling for a few companies every 15 minutes just pull the top 100
        - news expires fast so if no signal change then we leave it at that
        - use backfill for model training
     */
    let news: Vec<FmpNews> = client
        .get("https://financialmodelingprep.com/stable/news/stock")
        .query(&[
            ("from", start_date),
            ("symbols", tickers),  // Batch query
            ("apikey", api_key),
            ("limit", "100"),
            ("page", "0"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    
    Ok(news)
}

// backfill list of tickers
pub async fn backfill_fmp_news(
    client: &Client,
    tickers: &str,
    start_date: &str,
    end_date: Option<&str>,
    api_key: &str,
) -> Result<Vec<FmpNews>> {
    let mut all_news = Vec::new();
    
    // per ticker we pull all pages
    for ticker in tickers.split(',') {
        let ticker = ticker.trim();
        let mut page = 0;
        
        loop {
            let mut req = client
                .get("https://financialmodelingprep.com/stable/news/stock")
                .query(&[
                    ("from", start_date),
                    ("symbols", ticker),
                    ("apikey", api_key),
                    ("limit", "250"),
                    ("page", &page.to_string()),
                ]);
            
            if let Some(to) = end_date {
                req = req.query(&[("to", to)]);
            }
            
            let batch: Vec<FmpNews> = req
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            
            if batch.is_empty() {
                break;  // No more data for this ticker
            }
            
            //println!("{}: Page {} - {} records", ticker, page, batch.len());
            
            let batch_len = batch.len();
            all_news.extend(batch);
            
            if batch_len < 250 {
                break;  // Last page for this ticker
            }
            
            page += 1;
        }
    }
    
    Ok(all_news)
}
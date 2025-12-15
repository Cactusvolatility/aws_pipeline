use crate::{api, config::Config, models::*, aws};
use anyhow::{Result,bail};
use aws_config::BehaviorVersion;
use futures::future::Shared;
use futures::stream:: {self, StreamExt};
use reqwest::Client as HttpClient;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_s3::Client as S3Client;
use chrono::Utc;
use chrono_tz::America::New_York;


#[derive(Clone)]
pub struct App {
    pub clients: Clients,
    pub config: Config,
}

#[derive(Clone)]
pub struct Clients {
    pub http: reqwest::Client,
    pub dynamo: aws_sdk_dynamodb::Client,
    pub s3: aws_sdk_s3::Client,
}

impl App {
    pub async fn new_from_env() -> Result<Self> {
        let config = Config::from_env()?;
        let shared = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let http = HttpClient::new();
        let dynamo = DynamoClient::new(&shared);
        let s3 = S3Client::new(&shared);

        Ok(Self {
            clients: Clients { http, dynamo, s3 },
            config,
        })
    }

    // Tiingo Methods

    pub async fn run_minute(&self) -> Result<()> {

        let books = api::tiingo::fetch_book(
            &self.clients.http, 
            &self.config.batch_tickers, 
            &self.config.tiingo_api_key,
        ).await?;

        if !books.is_empty() {
            aws::s3::write_book_s3(
                &self.clients.s3, 
                &self.config.s3_bucket, 
                &books,
            ).await?;
        }
        
        Ok(())
    }
    

    pub async fn run_ingest(&self, start_date: String, end_date: Option<String>) -> Result<()> {
        let results: Vec<_> = stream::iter(self.config.tickers.clone())
            .map(|ticker| {
                let app = self.clone();
                let start = start_date.clone();
                let end = end_date.clone();
                async move {
                    let result = app.process_ticker(&ticker, start, end).await;
                    (ticker, result)
                }
            })
            .buffer_unordered(self.config.max_concurrency)
            .collect()
            .await;

        let mut all_bars = Vec::new();
        let mut successes = Vec::new();

        for (ticker, result) in results {
            match result {
                Ok((bars, last_ts)) => {
                    all_bars.extend(bars);
                    successes.push((ticker, last_ts));
                }

                Err(e) => {
                    eprintln!("Ticker {} failed: {}", ticker, e);
                    /*
                    // Try to update failure state, but don't fail Lambda if this fails
                    if let Err(e2) = aws::dynamo::update_on_failure(
                        &self.clients.dynamo,
                        &self.config.dynamo_table,
                        &ticker,
                        &e.to_string(),
                    )
                    .await
                    {
                        eprintln!("Failed dynamo update failure state for {}: {}", ticker, e2);
                    }
                    */
                }
            }
        }
        
        // Write successful bars to S3
        if !all_bars.is_empty() {
            aws::s3::write_tiingo_s3(
                &self.clients.s3,
                &self.config.s3_bucket,
                &all_bars,
            )
            .await?;
        }
        
        // Update DynamoDB for successful tickers
        /* 
        for (ticker, last_ts) in successes {
            if let Err(e)  = aws::dynamo::save_next_ts(
                &self.clients.dynamo,
                &self.config.dynamo_table,
                &ticker,
                &last_ts,
            )
            .await
            {
                eprintln!("failed on saving timestamp for {} : {}", ticker, e);
            }
        }
        */
        
        Ok(())
    }
    

    async fn process_ticker(
        &self, 
        ticker: &str, 
        start_date: String, 
        end_date: Option<String>
    ) -> Result<(Vec<TickerBar>, Option<String>)> {
        
        println!("Process_ticker: fetching_intraday");

        let api_bars = api::tiingo::fetch_backfill(
                &self.clients.http,
                ticker,
                &start_date,
                end_date.as_deref(),
                &self.config.tiingo_api_key,
            )
        .await?;  // ← Error propagates up as Result::Err
        
        // TODO
            // consider Option<String>?
        if api_bars.is_empty() {
            return Ok((Vec::new(), Some("backfill result is empty".to_string())));
        }
        
        println!("process_ticker: converting from TiingoBar to TickerBar");
        // Transform to TickerBar
        let ticker_bars: Vec<TickerBar> = api_bars
            .into_iter()
            .map(|b| TickerBar {
                ticker: ticker.to_string(),
                date: b.date,
                open: b.open,
                high: b.high,
                low: b.low,
                close: b.close,
                volume: b.volume,
            })
            .collect();
        
        Ok((ticker_bars, None))
    }

    // FMP methods
    pub async fn run_fmp_news(&self) -> Result<()> {
        let tickers: Vec<String> = self.config.tickers.clone();

        let mut all_news = Vec::new();
        let today = Utc::now().with_timezone(&New_York).format("%Y-%m-%d").to_string();

        // split tickers into batches of 10
        
        for batch in tickers.chunks(10) {
            let symbols = batch.join(",");

            let news = api::fmp::fetch_fmp_news(
                &self.clients.http,
                &symbols,
                &today,
                &self.config.fmp_api_key,
            )
            .await?;

            all_news.extend(news);
        }

        if !all_news.is_empty() {
            aws::s3::write_fmp_news_s3(
                &self.clients.s3,
                &self.config.s3_bucket,
                &all_news,
            )
            .await?;
        }

        Ok(())
    }

}
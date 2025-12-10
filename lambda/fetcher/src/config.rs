use anyhow::Result;

#[derive(Clone)]
pub struct Config {
    pub tiingo_api_key: String,
    pub fmp_api_key: String,
    pub tickers: Vec<String>,
    pub dynamo_table: String,
    pub s3_bucket: String,
    pub max_concurrency: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        println!("start app construct");
        Ok(Self {
            tiingo_api_key: std::env::var("TIINGO_API_KEY")?,
            fmp_api_key: std::env::var("FMP_API_KEY")?,
            tickers: parse_tickers()?,
            dynamo_table: std::env::var("DYNAMODB_TABLE")?,
            s3_bucket: std::env::var("S3_BUCKET")?,
            max_concurrency: 10,
        })
    }
}

fn parse_tickers() -> Result<Vec<String>> {
    let tickers = std::env::var("TICKERS")?;

    let mut tickers: Vec<String> = tickers
        .split(',')
        .map(|ticker| ticker.trim().to_string())
        .filter(|ticker| !ticker.is_empty())
        .collect();

    tickers.dedup();
    Ok(tickers)
}
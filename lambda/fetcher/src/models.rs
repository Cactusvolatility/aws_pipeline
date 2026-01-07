use serde::{Deserialize, Serialize};
/*
    use to store different structs for data sources
*/

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct BackfillEvent {
    pub mode: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct TiingoBar {
    pub date: String,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct TickerBar {
    pub ticker: String,
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// CamelCase
#[derive(Debug, Deserialize)]
pub struct TiingoBook {
    pub ticker: String,
    pub timestamp: String,
    #[serde(default)]
    pub lastSaleTimestamp: Option<String>,
    #[serde(default)]
    pub quoteTimestamp: Option<String>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    #[serde(default)]
    pub mid: Option<f64>,
    pub tngoLast: f64,
    #[serde(default)]
    pub last: Option<f64>,
    #[serde(default)]
    pub lastSize: Option<i64>,
    #[serde(default)]
    pub bidSize: Option<i64>,
    #[serde(default)]
    pub bidPrice: Option<f64>,
    #[serde(default)]
    pub askPrice: Option<f64>,
    #[serde(default)]
    pub askSize: Option<i64>,
    pub volume: i64,
    pub prevClose: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FmpNews {
    pub symbol: String,
    pub publishedDate:String,
    pub publisher:String,
    pub title:String,
    #[serde(default)]
    pub image: Option<String>,
    pub site:String,
    pub text:String,
    pub url:String, 
}


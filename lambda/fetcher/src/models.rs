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


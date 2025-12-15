use aws_config::{imds::client, BehaviorVersion};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::Value;
use std::sync::Arc;
use chrono::{NaiveDate, Utc};

use fetcher::models::BackfillEvent;
use fetcher::app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = app::App::new_from_env()
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    
    
    println!("app loaded, FMP running");

    // just ref, don't use Arc
    run(service_fn(|event| handler(event, &app))).await
}

async fn handler(
    _event: LambdaEvent<Value>,
    app: &app::App,
) -> Result<Value, Error> {
    let payload: BackfillEvent = serde_json::from_value(_event.payload)
        .unwrap_or_default();

    if let Some(start_date) = payload.start_date {
        println!("provided start date - running backfill");
        let start = parse_date(&start_date)?;
        let today = Utc::now().date_naive();

        if start > today {
            return Err("start must be <= to today".into())
        }
    
        if let Some(ref end_date) = payload.end_date {
            let end = parse_date(end_date)?;

            if end < start {
                return Err("end must be after start".into());
            }

            if end > today {
                return Err("end must be before today".into())
            }
        }
        println!("fmp backfill not available yet");
        //app.run_ingest(start_date, payload.end_date).await?;
    }
    else {
        //println!("implement run_minute");
        app.run_fmp_news().await?;
    }

    Ok(serde_json::json!({"status": "FMP success"}))
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date format: {s}. FMP needs YYYY-MM-DD"))
}
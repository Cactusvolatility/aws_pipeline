use aws_config::{imds::client, BehaviorVersion};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::Value;
use std::sync::Arc;
use chrono::NaiveDate;

use crate::models::BackfillEvent;

//use fetcher::aws::{write_concurrent};
//use fetcher::api::{Request, Response, backfill_quotes, fetch_batch_quotes};
mod models;
mod api;
mod aws;
mod app;
mod config;

// See:
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_dynamodb_code_examples.html#serverless_examples
    // https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/client/struct.Client.html#method.batch_write_item


#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = app::App::new_from_env()
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    
    
    println!("app loaded, running");

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
        parse_date(&start_date)?;
    
        if let Some(ref end_date) = payload.end_date {
            parse_date(end_date)?;
        }
        app.run_ingest(start_date, payload.end_date).await?;
    }
    else {
        println!("implement run_minte");
        //app.run_minute().await?;
    }

    Ok(serde_json::json!({"status": "success"}))
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date format: {s}. Tiingo needs YYYY-MM-DD"))
}


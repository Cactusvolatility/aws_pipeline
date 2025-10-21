use aws_config::{imds::client, BehaviorVersion};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_sdk_dynamodb::{self as dynamodb, types::{AttributeValue, Put, PutRequest, WriteRequest}, Client as DynamoClient};
use anyhow::{Ok, Result};

use fetcher::aws::{write_concurrent};
use fetcher::api::{Request, Response, backfill_quotes, fetch_batch_quotes};

// See:
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_dynamodb_code_examples.html#serverless_examples
    // https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/client/struct.Client.html#method.batch_write_item



async fn function_handler(
    _event: LambdaEvent<Request>,
    ddb: DynamoClient,
    table: &str,
) -> Result<()> {

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let api_key = std::env::var("TIINGO_API_KEY")
        .expect("api key missing");

    // test - if empty will go to AAPL - else MSFT
    let tickers_str = std::env::var("TICKERS")
        .unwrap_or_else(|_| "AAPL".to_string());

    let tickers: Vec<String> = tickers_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let payload = _event.payload;
    match payload.mode.as_deref(){
        Some("backfill") => {
            let all_quotes = backfill_quotes(payload, &api_key).await?;
            // write (one item per day)
            write_concurrent(&ddb, table, all_quotes, /*max_concurrency*/ 4, &today).await?;
        }
        _ => {
            let quotes = fetch_batch_quotes(&tickers, 5, 4, &api_key).await?;

            write_concurrent(&ddb, table, quotes, 4, &today).await?;
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    println!("Hello Rust 456 789");
    println!("echo 012");

    std::env::set_var("RUST_BACKTRACE", "1");
    std::panic::set_hook(Box::new(|p| eprintln!("why am I panicing {p:#?}")));
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if let Err(e) = fmt().with_env_filter(filter).with_target(false).without_time().try_init() {
        eprintln!("TRACE INIT FAILED (non-fatal): {e}");
    }
    
    /*tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();*/

    tracing::info!("init lambda");
    println!("tracing on");
    let table_name = std::env::var("DYNAMODB_TABLE")
        .expect("DYNAMODB_TABLE environment var not set");

    let cfg = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let ddb = DynamoClient::new(&cfg);

    println!("go to custom_function");
    // ??? anyhow is giving me issues
    let custom_func = service_fn(|event: LambdaEvent<Request>| {
        let ddb = ddb.clone();
        let table_name = table_name.clone();
        async move { function_handler(event, ddb, &table_name).await
            .map_err(|e| -> lambda_runtime::Error {e.into()})
        }
    });

    println!("ran custom func");
    run(custom_func).await

}

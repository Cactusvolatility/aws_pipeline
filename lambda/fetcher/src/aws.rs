use std::{collections::HashMap, time::Duration};
use anyhow::{bail, Result};
use aws_sdk_dynamodb::{
    types::{AttributeValue, PutRequest, WriteRequest}, 
    Client as DynamoClient
};
use futures::{stream::FuturesUnordered, StreamExt};
use rand::Rng;
use tokio::time::sleep;
use crate::api::{Quote};

pub fn quote_to_item(q: &Quote) -> HashMap<String, AttributeValue> {
    HashMap::from([
        ("ticker".into(), AttributeValue::S(q.ticker.clone())),
        ("date".into(), AttributeValue::S(q.date.to_string())),
        ("price".into(), AttributeValue::N(q.close.to_string())),
        ("volume".into(), AttributeValue::N(q.volume.to_string())),
    ])
}


// see:
    // https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/types/builders/struct.PutRequestBuilder.html

async fn write_batch_singular(
    ddb: DynamoClient,
    table: String,
    items: Vec<HashMap<String, AttributeValue>>,
) -> Result<()> {
    let mut writes: Vec<WriteRequest> = Vec::with_capacity(items.len());

    for item in items {
        let put = PutRequest::builder()
            .set_item(Some(item.clone()))
            .build()?;

        // build in putrequest gives a result but write request build does not (?)

        let wr = WriteRequest::builder()
            .put_request(put)
            .build();

        writes.push(wr);
    }

    let mut request_items: HashMap<String, Vec<WriteRequest>> = HashMap::from([(table.to_string(), writes)]);

    // simple backoff when I need to re-attempt
    let mut backoff = Duration::from_millis(100);
    let max_backoff = Duration::from_millis(4_000);
    let mut attempts = 0u32;
    let max_attempts = 8;

    loop {
        attempts += 1;

        let resp = ddb
            .batch_write_item()
            .set_request_items(Some(request_items))
            .send()
            .await?;

        match resp.unprocessed_items {
            Some(unprocessed) if !unprocessed.is_empty() => {
                let leftover: usize = unprocessed
                    .values()
                    .map(|val| val.len())
                    .sum();
                tracing::warn!(
                    "leftover requests, {leftover} unprocessed writes at {attempts} attempt"
                );

                if attempts >= max_attempts {
                    return bail!( "Exceeded retry attempts, {} writes leftover", leftover)
                }

                let backoff_ms = (backoff.as_millis() as u64) * (2_u64.pow(attempts-1));
                let cap_backoff = backoff_ms.min(max_backoff.as_millis() as u64);

                let jitter_ms = Duration::from_millis(rand::rng().random_range(100..=cap_backoff));
                tracing::info!("Backoff for {:?}", jitter_ms);
                sleep(jitter_ms).await;
                request_items = unprocessed;
            }

            _ => {
                tracing::info!("Nothing unproccsed.");
                break
            },
        }


    }
    Ok(())
}

// handle uneven lengths in case
fn chunked<T:Clone>(xs: &[T], n: usize) -> Vec<Vec<T>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < xs.len() {
        let end = (i + n).min(xs.len());
        out.push(xs[i..end].to_vec());
        i = end;
    }

    out
}

// each async block is a new type - so it's fighting me?
pub async fn write_concurrent(
    ddb: &DynamoClient,
    table: &str,
    quotes: Vec<Quote>,
    max_concurrency: usize,
    date: &str,
) -> Result<()> {
    assert!(max_concurrency > 0);

    let mut items = Vec::with_capacity(quotes.len());

    for quote in quotes {
        items.push(quote_to_item(&quote));
    }

    let batches = chunked(&items, 5);

    let mut in_flight = FuturesUnordered::new();
    let mut next = 0usize;

    while next < batches.len() && in_flight.len() < max_concurrency {
        let ddb_clone = ddb.clone();
        let table_owned = table.to_string();
        let batch = batches[next].clone();
        // futures unordered needs the future and not the result
            // remove .await
        in_flight.push(
            write_batch_singular(ddb_clone, table_owned, batch)
        );

        next += 1;
    }

    while let Some(res) = in_flight.next().await {
        res?;

        if next < batches.len() {
            let ddb_clone = ddb.clone();
            let table_owned = table.to_string();
            let batch = batches[next].clone();
            in_flight.push(
                write_batch_singular(ddb_clone, table_owned, batch)
            );

            next += 1;
        }
    }
    println!("finish concurrent batch write");

    Ok(())
}
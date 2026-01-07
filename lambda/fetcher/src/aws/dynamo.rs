use anyhow::{Result, anyhow};
use aws_sdk_dynamodb::{types::AttributeValue, Client, error::SdkError, operation::put_item::PutItemError};
use chrono::{Utc, NaiveDateTime};
use chrono_tz::America::New_York;
use std::collections::HashMap;
use crate::models::{FmpNews};

// load/save the dynamodb states

pub async fn load_next_ts(
    client: &Client,
    table: &str,
    ticker: &str,
) -> Result<String> {
    let resp = client
                                .get_item()
                                .table_name(table)
                                .key("ticker", AttributeValue::S(ticker.to_string()))
                                .send().await?;

    if let Some(item) = resp.item {
        if let Some(value) = item.get("next_start_ts") {
            if let Ok(time) = value.as_s() {
                return Ok(time.to_string());
            }
        }
    }

    Ok("2025-01-01T00:00:00Z".to_string())
}

pub async fn save_next_ts(
    client: &Client,
    table: &str,
    ticker: &str,
    next_ts: &str,
) -> Result<()> {

    client
        .put_item()
        .table_name(table)
        .item("ticker", AttributeValue::S(ticker.to_string()))
        .item("next_start_ts", AttributeValue::S(next_ts.to_string()))
        .item("last_updated", AttributeValue::S(Utc::now().to_rfc3339()))
        .item("consecutive_failures", AttributeValue::N("0".to_string()))
        .item("last_error", AttributeValue::Null(true))
        .send()
        .await?;

    Ok(())
}

//TODO
    // 12/5 basic failure setup
    // add additional rows as needed

pub async fn update_on_failure(
    client: &Client,
    table: &str,
    ticker: &str,
    error_msg: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    
    client
        .update_item()
        .table_name(table)
        .key("ticker", AttributeValue::S(ticker.to_string()))
        .update_expression(
            "SET last_error = :err, last_updated = :now \
             ADD consecutive_failures :one"
        )
        .expression_attribute_values(":err", AttributeValue::S(error_msg.to_string()))
        .expression_attribute_values(":now", AttributeValue::S(now))
        .expression_attribute_values(":one", AttributeValue::N("1".to_string()))
        .send()
        .await?;
    
    Ok(())
}

pub async fn write_articles(
    client: &Client,
    table_name: &str,
    articles: &[FmpNews],
) -> Result<()> {
    
    let mut new_count = 0;
    let mut duplicate_count = 0;
    
    for article in articles {
        let mut item = HashMap::new();
        let published_datetime = NaiveDateTime::parse_from_str(
            &article.publishedDate, 
            "%Y-%m-%d %H:%M:%S"
        )?
        .and_utc();

        item.insert("article_id".to_string(), AttributeValue::S(article.url.clone()));
        item.insert("url".to_string(), AttributeValue::S(article.url.clone()));
        item.insert("title".to_string(), AttributeValue::S(article.title.clone()));
        item.insert("ticker".to_string(), AttributeValue::S(article.symbol.clone()));
        item.insert("published_at".to_string(), AttributeValue::N(
            published_datetime.timestamp().to_string()
        ));
        let ttl = (chrono::Utc::now() + chrono::Duration::days(7)).timestamp();
        item.insert("ttl".to_string(), AttributeValue::N(ttl.to_string()));
        
        let result = client
            .put_item()
            .table_name(table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(article_id)")
            .send()
            .await;
        
        match result {
            Ok(_) => new_count += 1,
            Err(SdkError::ServiceError(err)) => {
                match err.err() {
                    PutItemError::ConditionalCheckFailedException(_) => {
                        duplicate_count += 1;
                    }
                    _ => return Err(anyhow!("DynamoDB service error: {:?}", err)),
                }
            }
            Err(e) => return Err(anyhow!("DynamoDB SDK error: {:?}", e)),
        }
    }
    
    println!("DynamoDB: {} new articles, {} duplicates skipped", new_count, duplicate_count);
    Ok(())
}
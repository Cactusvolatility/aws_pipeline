use anyhow::Result;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::Utc;

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
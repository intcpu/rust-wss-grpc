use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct BnBookTickerMessage {
    stream: String,
    data: BnBookTickerData,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct BnBookTickerData {
    #[serde(rename = "e")]
    event: String,
    #[serde(rename = "u")]
    id: u64,
    #[serde(rename = "s")]
    pair: String,
    #[serde(rename = "b")]
    bid_price: String,
    #[serde(rename = "B")]
    bid_size: String,
    #[serde(rename = "a")]
    ask_price: String,
    #[serde(rename = "A")]
    ask_size: String,
    #[serde(rename = "T")]
    trade_time: u64,
    #[serde(rename = "E")]
    event_time: u64,
}

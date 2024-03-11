use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct BnBookTickerMessage {
    pub(crate) stream: String,
    pub(crate) data: BnBookTickerData,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct BnBookTickerData {
    #[serde(rename = "e")]
    pub(crate) event: String,
    #[serde(rename = "u")]
    pub(crate) id: u64,
    #[serde(rename = "s")]
    pub(crate) pair: String,
    #[serde(rename = "b")]
    pub(crate) bid_price: String,
    #[serde(rename = "B")]
    pub(crate) bid_size: String,
    #[serde(rename = "a")]
    pub(crate) ask_price: String,
    #[serde(rename = "A")]
    pub(crate) ask_size: String,
    #[serde(rename = "T")]
    pub(crate) trade_time: u64,
    #[serde(rename = "E")]
    pub(crate) event_time: u64,
}

use reqwest::Client;
use tracing::error;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BinanceResponse<T: Sized> {
    symbols: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct UsdtMarginMarket {
    symbol: String,
    pair: String,
    contractType: String,
    deliveryDate: u64,
    onboardDate: u64,
    status: String,
    maintMarginPercent: String,
    requiredMarginPercent: String,
    baseAsset: String,
    quoteAsset: String,
    marginAsset: String,
    pricePrecision: i64,
    quantityPrecision: i64,
    baseAssetPrecision: i64,
    quotePrecision: i64,
    underlyingType: String,
    triggerProtect: String,
    filters: Vec<HashMap<String, Value>>,
    orderTypes: Vec<String>,
    timeInForce: Vec<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// see <https://binance-docs.github.io/apidocs/futures/en/#exchange-information>
pub(crate) async fn um_get_symbols() -> Result<Vec<String>, ()> {
    let markets = http_um_get_markets().await.unwrap();
    let symbols: Vec<String> = markets.into_iter().map(|m| m.symbol).collect();
    Ok(symbols)
}

pub(crate) async fn http_um_get_markets() -> Result<Vec<UsdtMarginMarket>, ()> {
    // 创建一个 reqwest 客户端
    let client = Client::new();

    // 发送异步 GET 请求并获取响应
    let response = match client
        .get("https://fapi.binance.com/fapi/v1/exchangeInfo")
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to send request: {}", e);
            return Err(());
        }
    };

    // 检查响应是否成功
    if response.status().is_success() {
        // 将响应的 JSON 数据解析为字符串
        let body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                error!("Failed to read response body: {}", e);
                return Err(());
            }
        };
        let resp = match serde_json::from_str::<BinanceResponse<UsdtMarginMarket>>(&body) {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
                return Err(());
            }
        };
        let symbols: Vec<UsdtMarginMarket> = resp
            .symbols
            .into_iter()
            .filter(|m| m.status == "TRADING")
            .collect();
        return Ok(symbols);
    } else {
        error!("Failed to get data: {:?}", response.status());
        return Err(());
    }
}

use chrono::Local;
use serde_json::json;
use std::borrow::Cow;
use tracing::{error, info, Level};
use tracing_subscriber::util::SubscriberInitExt;
use ws_tool::frame::OpCode;
use ws_tool::{
    codec::AsyncStringCodec,
    connector::{async_tcp_connect, async_wrap_rustls, get_host},
    ClientBuilder,
};

pub async fn bn_wss_bookticker(pair: &str) -> Result<(), ()> {
    let pair_lower = pair.to_lowercase().replace("_", "");
    println!("{}", pair_lower); // 输出: btcusdt

    let uri: http::Uri = format!(
        "wss://fstream-mm.binance.com/stream?streams={}@bookTicker",
        pair_lower
    )
    .parse()
    .unwrap();
    info!("{:?}", uri);
    let builder = ClientBuilder::new();
    let stream = match async_tcp_connect(&uri).await {
        Ok(stream) => stream,
        Err(err) => {
            error!("Failed to connect to WebSocket: {}", err);
            return Err(());
        }
    };
    let stream = match async_wrap_rustls(stream, get_host(&uri).unwrap(), vec![]).await {
        Ok(stream) => stream,
        Err(err) => {
            error!("Failed to wrap WebSocket with TLS: {}", err);
            return Err(());
        }
    };
    let mut client = match builder
        .async_with_stream(uri.clone(), stream, AsyncStringCodec::check_fn)
        .await
    {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to create WebSocket client: {}", err);
            return Err(());
        }
    };

    while let Ok(msg) = client.receive().await {
        // info!("{}", msg.data.trim());
        let local_time = Local::now();
        println!("{}:{}", msg.data.trim(), local_time);
    }
    Ok(())
}

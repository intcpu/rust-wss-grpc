use chrono::Local;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::tungstenite::Message;
use tracing::Level;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
// use wss::bookticker;
// use wss::wss_test;
use crypto_ws_client::{BinanceLinearWSClient, BinanceSpotWSClient, WSClient};

// mod wss;

// pub mod signal {
//     tonic::include_proto!("signal");
// }

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::fmt()
    //     .with_max_level(Level::INFO)
    //     .finish()
    //     .try_init()
    //     .expect("failed to init log");
    // let pair = "XRP_USDT";
    // println!("------end-----");
    // // bookticker::bn_wss_bookticker(pair).await;
    // wss_test::wss_test().await.expect("TODO: panic message");

    let (tx, rx) = std::sync::mpsc::channel();
    tokio::task::spawn(async move {
        let symbols = vec![
            "XRPUSDT".to_string().to_lowercase(),
            // "ETHUSDT".to_string().to_lowercase(),
        ];
        let ws_client = BinanceLinearWSClient::new(tx, None).await;
        ws_client.subscribe_trade(&symbols).await;
        ws_client.run().await
        // run for 5 seconds
        // let _ = tokio::time::timeout(std::time::Duration::from_secs(5), ws_client.run()).await;
        // ws_client.close();
    });
    tokio::task::spawn(async move {
        for msg in &rx {
            let local_time = Local::now();
            println!("{:?} {}", msg, local_time)
        }
    });

    tokio::task::spawn(async move {
        let addr = "fstream.binance.com";
        // let tcp_url = format!("{}:443", addr);
        let url = format!("wss://{}/stream?streams={}@bookTicker", addr, "xrpusdt");

        // let stream = TcpStream::connect(&tcp_url).await?;

        // println!("stream : {stream:?}");
        println!("Connected to {:?}", addr);
        println!("{url:?}");

        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        let (write, mut read) = ws_stream.split();

        while let Some(message) = read.next().await {
            match message {
                Ok(data) => {
                    // let string = String::from_utf8_lossy(&data.into_data());
                    let local_time = Local::now();
                    println!("{} {}", data, local_time);
                }
                Err(err) => {
                    eprintln!("Error reading message: {}", err);
                }
            }
        }
    });

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

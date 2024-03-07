use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde_json::json;

use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

type AnyError = Box<dyn std::error::Error + Send + Sync>;

pub async fn wss_test() -> Result<(), AnyError> {
    let addr = "fstream.binance.com";
    let tcp_url = format!("{}:443", addr);
    let url = format!("wss://{}/stream?streams={}@bookTicker", addr, "xrpusdt");

    let stream = TcpStream::connect(&tcp_url).await?;

    println!("stream : {stream:?}");
    println!("Connected to {:?}", addr);
    println!("{url:?}");

    if let Some(mut ws_stream) = client_async(&url, stream).await {
        match ws_stream {
            Ok(ws_stream) => ws_stream,
            Err(e) => {
                println!("Failed to connect to server: {:?}", e);
                return Err(e);
            }
        }
    }
    println!("Handshake successful.");
    Ok(())
    //
    // if let Some(item) = ws_stream.next().await {
    //     match item {
    //         Ok(msg) => match msg {
    //             Message::Text(text) => {
    //                 println!("Received text message: {}", text);
    //             }
    //             Message::Close(frame) => {
    //                 println!("Received close message: {:?}", frame);
    //
    //                 if let Err(e) = ws_stream.close(None).await {
    //                     match e {
    //                         WsError::ConnectionClosed => (),
    //                         _ => {
    //                             println!("Error while closing: {}", e);
    //                             break;
    //                         }
    //                     }
    //                 }
    //
    //                 println!("Sent close message.");
    //
    //                 println!("Closing...");
    //                 return Ok(());
    //             }
    //             _ => (),
    //         },
    //         Err(e) => {
    //             println!("Error receiving message: \n{0:?}\n{0}", e);
    //         }
    //     }
    // }
    //
    // ws_stream.close(None).await?;
    // Ok(())
}

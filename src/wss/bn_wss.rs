use futures::StreamExt;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

pub async fn bn_wss_bookticker(pair: &str, tx: broadcast::Sender<String>) -> Result<(), ()> {
    let pair_lower = pair.to_lowercase().replace("_", "");
    let addr = "fstream-mm.binance.com";
    let url = format!("wss://{}/stream?streams={}@bookTicker", addr, pair_lower);

    info!("Connected to addr: {:?}, url: {:?}", addr, url);

    let (ws_stream, _) = match tokio_tungstenite::connect_async(url).await {
        Ok(ws_stream) => ws_stream,
        Err(err) => {
            error!("Failed to connect to WebSocket: {}", err);
            return Err(());
        }
    };
    info!("WebSocket connection established");

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // info!("Received text message: {}", text_str);

                // let timestamp_millis = Utc::now().timestamp_millis();
                // println!("Received message: {:?} at {}", msg_json, timestamp_millis);

                if let Err(err) = tx.send(text) {
                    error!("Failed to send bn_wss_bookticker message: {:?}", err);
                    continue;
                }
            }
            Ok(Message::Binary(data)) => {
                error!("Received binary message with {} bytes", data.len());
                // Process the binary message
            }
            Ok(Message::Close(_)) => {
                error!("Received close message");
                return Err(());
                // Handle the close message
            }
            Ok(Message::Ping(data)) => {
                info!("Received ping message with {} bytes", data.len());
                // Handle the ping message
            }
            Ok(Message::Pong(data)) => {
                info!("Received pong message with {} bytes", data.len());
                // Handle the pong message
            }
            Err(err) => {
                error!("Failed to receive message from WebSocket: {}", err);
                return Err(());
            }
            _ => {
                error!("Unknown message type");
            }
        };
    }

    Ok(())
}

use rpc::rpc_server;
use tokio::sync::broadcast;
use tracing::{error, info, warn, Level};
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use wss::{bn_wss, bn_wss_type};
mod rpc;
mod wss;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .finish()
        .try_init()
        .expect("failed to init log");
    let (tx, mut rx) = broadcast::channel::<String>(1);
    let task_a = tokio::task::spawn(async move {
        // let symbols = vec![
        //     "XRPUSDT".to_string().to_lowercase(),
        //     // "ETHUSDT".to_string().to_lowercase(),
        // ];
        let symbol = "XRP_USDT";
        bn_wss::bn_wss_bookticker(symbol, tx).await
    });
    let task_b = tokio::task::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let msg_str = msg.as_str();
                    let msg_json: bn_wss_type::BnBookTickerMessage =
                        match serde_json::from_str(msg_str) {
                            Ok(msg_json) => msg_json,
                            Err(err) => {
                                error!("WebSocket msg json decode error: {}", err);
                                continue;
                            }
                        };
                    // let local_time = Local::now();
                    info!("---- ---- {:?}", msg_json)
                }
                Err(broadcast::error::RecvError::Closed) => {
                    error!("Channel closed");
                    return;
                }
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    warn!("Dropped {} lagged messages", count);
                    continue;
                }
            }
        }
    });

    let task_c = tokio::task::spawn(async move { rpc_server::rpc_server().await });

    let (r1, r2, r3) = tokio::join!(task_a, task_b, task_c);
    error!("{r1:?}, {r2:?}, {r3:?}");
}

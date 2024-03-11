use crate::rpc::rpc_server::signal::BookTickerResp;
use rpc::rpc_server;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
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
    let pair_bts: Arc<RwLock<HashMap<String, BookTickerResp>>> =
        Arc::new(RwLock::new(HashMap::new()));
    // let pair_bts_r = pair_bts.clone();
    let (tx, mut rx) = broadcast::channel::<String>(1);

    let task_a = tokio::task::spawn(async move {
        // let symbols = vec![
        //     "XRPUSDT".to_string().to_lowercase(),
        //     // "ETHUSDT".to_string().to_lowercase(),
        // ];
        let symbol = "XRP_USDT";
        bn_wss::bn_wss_bookticker(symbol, tx).await
    });
    let pair_bts_write = pair_bts.clone();
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
                    // info!("---- ---- {:?}", msg_json)
                    let mut pd = pair_bts_write.write().await; // 获取锁
                    let msg_data = BookTickerResp {
                        data: Default::default(),
                    };
                    let pair = "XRP_USDT".to_string();
                    pd.insert(pair, msg_data);
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    error!("Channel closed");
                    return;
                }
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    // warn!("Dropped {} lagged messages", count);
                    continue;
                }
            }
        }
    });
    let pair_bts_read = pair_bts.clone();
    let task_c = tokio::task::spawn(async move {
        if let Err(_) = rpc_server::rpc_server(pair_bts_read).await {
            error!("Failed to start RPC server");
        }
    });

    let (r1, r2, r3) = tokio::join!(task_a, task_b, task_c);

    error!("{r1:?}, {r2:?}, {r3:?}");
}

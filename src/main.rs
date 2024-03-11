use crate::rpc::rpc_server::signal::{MarginType, PairBookTicker};
use crate::rpc::rpc_types::{AllBookTickers, SpotBookTickers, UsdtMarginBookTickers};
// use chrono::Local;
use dashmap::DashMap;
use rpc::rpc_server;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, Level};
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use wss::{bn_wss, bn_wss_type};

// mod rest;
mod rpc;
mod wss;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .finish()
        .try_init()
        .expect("failed to init log");
    // let symbol_data = rest::bn_rest::um_get_symbols().await.unwrap();
    // println!("{:?}", symbol_data);

    let spot_book_tickers = SpotBookTickers {
        data: Arc::new(DashMap::with_capacity(256)),
    };
    let usdt_margin_book_tickers = UsdtMarginBookTickers {
        data: Arc::new(DashMap::with_capacity(256)),
    };
    let all_book_tickers = AllBookTickers {
        spot: SpotBookTickers {
            data: spot_book_tickers.data.clone(),
        },
        usdt_margin: UsdtMarginBookTickers {
            data: usdt_margin_book_tickers.data.clone(),
        },
    };

    info!(" Starting Binance WebSocket Server Task");
    let (tx, mut rx) = broadcast::channel::<String>(1);
    let task_a = tokio::task::spawn(async move {
        // let symbols = vec![
        //     "XRPUSDT".to_string().to_lowercase(),
        //     // "ETHUSDT".to_string().to_lowercase(),
        // ];
        let symbol = "BTCUSDT";
        bn_wss::bn_um_wss_bookticker(symbol, tx).await
    });

    info!(" Starting WebSocket Msg Update Task");
    let pair_bts_write = usdt_margin_book_tickers.data.clone();
    let task_b = tokio::task::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    // let local_time = Local::now();
                    // println!("---- 1 ---- {:?}", local_time);
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
                    // println!(
                    //     "---- 2 ---- {:?} {:?}",
                    //     local_time,
                    //     msg_json.data.event_time.clone()
                    // );
                    let msg_margin_type = MarginType::UsdtMargin;
                    let msg_data = PairBookTicker {
                        margin_type: i32::from(msg_margin_type),
                        timestamp: msg_json.data.event_time.clone(),
                        bid: msg_json.data.bid_price.parse().unwrap(),
                        ask: msg_json.data.ask_price.parse().unwrap(),
                    };
                    let pair = msg_json.data.pair.clone().to_string();
                    pair_bts_write.insert(pair, msg_data);
                    // let local_time = Local::now();
                    // println!(
                    //     "---- 3 ---- {:?} {:?}",
                    //     local_time,
                    //     msg_json.data.event_time.clone()
                    // );
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    error!("Channel closed");
                    return;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // warn!("Dropped {} lagged messages", count);
                    continue;
                }
            }
        }
    });

    info!("Starting RPC server Task");
    let task_c = tokio::task::spawn(async move {
        if let Err(_) = rpc_server::rpc_server(all_book_tickers).await {
            error!("Failed to start RPC server");
            process::exit(1);
        }
    });

    info!("all task is join");
    let (r1, r2, r3) = tokio::join!(task_a, task_b, task_c);

    error!("all task end : {r1:?}, {r2:?}, {r3:?}");
}

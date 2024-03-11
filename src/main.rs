use crate::rpc::rpc_server::signal::{MarginType, PairBookTicker};
use crate::rpc::rpc_types::{AllBookTickers, SpotBookTickers, UsdtMarginBookTickers};
use std::collections::HashMap;
// use chrono::Local;
use chrono::Local;
use dashmap::DashMap;
use rpc::rpc_server;
use std::process;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tracing::{error, info, Level};
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use wss::{bn_wss, bn_wss_type};

mod rest;
mod rpc;
mod wss;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .finish()
        .try_init()
        .expect("failed to init log");
    let symbols = match rest::bn_rest::um_get_symbols().await {
        Ok(symbols) => symbols,
        Err(e) => {
            error!("Failed to get symbols: {:?}", e);
            process::exit(1);
        }
    };

    let wss_channels: Arc<Mutex<HashMap<String, (Sender<String>, Receiver<String>)>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let spot_book_tickers = SpotBookTickers {
        data: Arc::new(DashMap::with_capacity(512)),
    };
    let usdt_margin_book_tickers = UsdtMarginBookTickers {
        data: Arc::new(DashMap::with_capacity(512)),
    };
    let all_book_tickers = AllBookTickers {
        spot: SpotBookTickers {
            data: spot_book_tickers.data.clone(),
        },
        usdt_margin: UsdtMarginBookTickers {
            data: usdt_margin_book_tickers.data.clone(),
        },
    };

    {
        info!(" Starting Binance WebSocket Server Task");
        let mut wss_channels = wss_channels.lock().await;
        for symbol in symbols.clone() {
            wss_channels.insert(symbol.clone(), broadcast::channel::<String>(1));
        }
    }

    let pair_bts_write = usdt_margin_book_tickers.data.clone();
    let wss_channels_clone = Arc::clone(&wss_channels);
    let receiver_task = tokio::task::spawn(async move {
        let wss_channels = wss_channels_clone.lock().await;
        let mut tasks: Vec<JoinHandle<()>> = vec![];
        for (symbol, (tx, rx)) in wss_channels.iter() {
            let mut rx = tx.subscribe();
            let pair_bts_write = pair_bts_write.clone();
            let symbol = symbol.clone();
            let task = tokio::task::spawn(async move {
                // info!(
                //     " {} usdt margin wss msg update: tx{:?} rx{:?}",
                //     symbol, tx, rx
                // );
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
                            //     "---- 3 ---- {} {:?} {:?}",
                            //     symbol,
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
            tasks.push(task);
        }
        drop(wss_channels);
        for task in tasks.into_iter() {
            task.await.unwrap();
        }
        // tasks.clear();
    });

    let wss_channels_clone = Arc::clone(&wss_channels);
    let sender_task = tokio::task::spawn(async move {
        let wss_channels = wss_channels_clone.lock().await;
        let mut tasks: Vec<JoinHandle<()>> = vec![];
        for (symbol, (tx, _)) in wss_channels.iter() {
            let symbol = symbol.clone();
            let tx = tx.clone();
            let task = tokio::task::spawn(async move {
                // info!(" {} usdt margin wss start: tx{:?}", symbol, &tx);
                tokio::task::spawn(async move { bn_wss::bn_um_wss_bookticker(&symbol, tx).await });
            });
            tasks.push(task);
        }
        drop(wss_channels);
        for task in tasks.into_iter() {
            task.await.unwrap();
        }
        // tasks.clear();
    });

    info!("Starting RPC server Task");
    let task_c = tokio::task::spawn(async move {
        if let Err(_) = rpc_server::rpc_server(all_book_tickers).await {
            error!("Failed to start RPC server");
            process::exit(1);
        }
    });

    info!("all task is join");
    let (r1, r2, r3) = tokio::join!(receiver_task, sender_task, task_c);

    error!("all task end : {r1:?} {r2:?} {r3:?}");
}

use crate::rpc::rpc_server::signal::{MarginType, PairBookTicker};
use crate::rpc::rpc_types::{AllBookTickers, SpotBookTickers, UsdtMarginBookTickers};
// use std::collections::HashMap;
// use chrono::Local;
use crate::rest::bn_rest::{
    http_spot_get_markets, http_um_get_markets, spot_get_symbols, um_get_symbols,
};
use chrono::Utc;
use dashmap::DashMap;
use rpc::rpc_server;
use std::process;
use std::sync::Arc;
// use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn, Level};
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

    let spot_markets = http_spot_get_markets().await.unwrap();
    // let spot_price_float = match spot_get_price_ticker_size(&spot_markets).await {
    //     Ok(d) => d,
    //     Err(e) => {
    //         error!("Failed to get spot_price_float: {:?}", e);
    //         process::exit(1);
    //     }
    // };

    let spot_symbols = match spot_get_symbols(&spot_markets).await {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to get spot symbols: {:?}", e);
            process::exit(1);
        }
    };

    let um_markets = http_um_get_markets().await.unwrap();
    // let um_price_float = match um_get_price_ticker_size(&um_markets).await {
    //     Ok(d) => d,
    //     Err(e) => {
    //         error!("Failed to get um_price_float: {:?}", e);
    //         process::exit(1);
    //     }
    // };

    let um_symbols = match um_get_symbols(&um_markets).await {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to get usdt margin symbols: {:?}", e);
            process::exit(1);
        }
    };

    let mut spot_tasks: Vec<JoinHandle<()>> = vec![];
    let spot_bts_write = spot_book_tickers.data.clone();
    for symbol in spot_symbols.iter() {
        let spot_bts_write = spot_bts_write.clone();
        let symbol = symbol.clone();
        let task = tokio::task::spawn(async move {
            let (tx, rx) = broadcast::channel::<String>(1);
            let symbol_t = symbol.clone();
            let symbol_r = symbol.clone();
            let tx_task = tokio::task::spawn(async move {
                loop {
                    info!("{:?} spot wss start", &symbol_t);
                    let tx = tx.clone();
                    let res = bn_wss::bn_spot_wss_bookticker(&symbol_t, tx).await;
                    error!("{:?} spot wss end:{:?}", &symbol_t, res);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            });
            let mut rx = rx.resubscribe();
            let rx_task = tokio::task::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(msg) => {
                            let msg_str = msg.as_str();
                            let msg_json: bn_wss_type::BnSpotBookTickerMessage =
                                match serde_json::from_str(msg_str) {
                                    Ok(msg_json) => msg_json,
                                    Err(err) => {
                                        error!(
                                            "{} spot WebSocket msg json decode error: {}",
                                            &symbol_r, err
                                        );
                                        continue;
                                    }
                                };
                            let now = Utc::now();
                            let microseconds: u64 = now.timestamp_millis() as u64;
                            let msg_margin_type = MarginType::Spot;
                            let msg_data = PairBookTicker {
                                margin_type: i32::from(msg_margin_type),
                                timestamp: microseconds,
                                bid: msg_json.data.bid_price.parse().unwrap(),
                                ask: msg_json.data.ask_price.parse().unwrap(),
                            };
                            let pair = msg_json.data.pair.clone().to_string();
                            spot_bts_write.insert(pair, msg_data);
                            // println!("---- 3 ---- {} {:?}", symbol_r, microseconds);

                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            error!("{} spot Channel closed", &symbol_r);
                            return;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            // warn!("Dropped {} lagged messages", count);
                            continue;
                        }
                    }
                }
            });
            let (r1, r2) = tokio::join!(tx_task, rx_task);
            error!("{:?} spot tx_task and rx_task end : {r1:?} {r2:?}", &symbol);
        });
        spot_tasks.push(task);
    }

    let mut usdt_tasks: Vec<JoinHandle<()>> = vec![];
    let usdt_bts_write = usdt_margin_book_tickers.data.clone();
    for symbol in um_symbols.iter() {
        let usdt_bts_write = usdt_bts_write.clone();
        let symbol = symbol.clone();
        let task = tokio::task::spawn(async move {
            let (tx, rx) = broadcast::channel::<String>(1);
            let symbol_t = symbol.clone();
            let symbol_r = symbol.clone();
            let tx_task = tokio::task::spawn(async move {
                loop {
                    info!("{:?} usdt margin wss start", &symbol_t);
                    let tx = tx.clone();
                    let res = bn_wss::bn_um_wss_bookticker(&symbol_t, tx).await;
                    error!("{:?} usdt margin wss end:{:?}", &symbol_t, res);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            });
            let mut rx = rx.resubscribe();
            let rx_task = tokio::task::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(msg) => {
                            let msg_str = msg.as_str();
                            let msg_json: bn_wss_type::BnUmBookTickerMessage =
                                match serde_json::from_str(msg_str) {
                                    Ok(msg_json) => msg_json,
                                    Err(err) => {
                                        error!(
                                            "{} usdt margin WebSocket msg json decode error: {}",
                                            &symbol_r, err
                                        );
                                        continue;
                                    }
                                };
                            let msg_margin_type = MarginType::UsdtMargin;
                            let msg_data = PairBookTicker {
                                margin_type: i32::from(msg_margin_type),
                                timestamp: msg_json.data.event_time.clone(),
                                bid: msg_json.data.bid_price.parse().unwrap(),
                                ask: msg_json.data.ask_price.parse().unwrap(),
                            };
                            let pair = msg_json.data.pair.clone().to_string();
                            usdt_bts_write.insert(pair, msg_data);
                            // let local_time = Local::now();
                            // let now = Utc::now();
                            // let microseconds: i128 = now.timestamp_millis() as i128;
                            // let event_time: i128 = msg_json.data.event_time.clone() as i128;
                            // if microseconds.checked_sub(event_time).unwrap_or_default() > 50 {
                            //     println!("---- 3 ---- {} {:?} {:?}", symbol, microseconds, event_time);
                            // }
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            error!("{} usdt margin Channel closed", &symbol_r);
                            return;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            // warn!("Dropped {} lagged messages", count);
                            continue;
                        }
                    }
                }
            });
            let (r1, r2) = tokio::join!(tx_task, rx_task);
            error!(
                "{:?} usdt margin tx_task and rx_task end : {r1:?} {r2:?}",
                &symbol
            );
        });
        usdt_tasks.push(task);
    }

    let rpc_task = tokio::task::spawn(async move {
        info!("Starting RPC server Task");
        if let Err(_) = rpc_server::rpc_server(all_book_tickers).await {
            error!("Failed to start RPC server");
            process::exit(1);
        }
    });

    let check_task = tokio::task::spawn(async move {
        info!("check all_book_tickers");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let spot_book = spot_book_tickers.data.clone();
            let usdt_book = usdt_margin_book_tickers.data.clone();
            let now = Utc::now();
            let microseconds: i128 = now.timestamp_millis() as i128;
            let time_out_set = 60000;
            info!(
                "spot_symbols len: {:?} spot_book_tickers len: {:?}",
                &spot_symbols.len(),
                &spot_book.len()
            );
            info!(
                "um_symbols len: {:?} usdt_margin_book_tickers: {:?}",
                &um_symbols.len(),
                &usdt_book.len()
            );
            for symbol in &spot_symbols {
                if let Some(a) = spot_book.get(symbol) {
                    let event_time: i128 = a.timestamp.clone() as i128;
                    let time_out = microseconds.checked_sub(event_time).unwrap_or_default();
                    if time_out > time_out_set {
                        warn!(
                            "{} spot book time from now: {:?} timeout: {:?}",
                            &symbol, microseconds, time_out
                        );
                    }
                } else {
                    error!("spot_book_tickers not found symbol: {:?}", &symbol)
                }
            }
            for symbol in &um_symbols {
                if let Some(a) = usdt_book.get(symbol) {
                    let event_time: i128 = a.timestamp.clone() as i128;
                    let time_out = microseconds.checked_sub(event_time).unwrap_or_default();
                    if time_out > time_out_set {
                        warn!(
                            "{} usdt margin book time from now: {:?} timeout: {:?}",
                            &symbol, microseconds, time_out
                        );
                    }
                } else {
                    error!("usdt_margin_book_tickers not found symbol: {:?}", &symbol)
                }
            }
        }
    });

    info!("all task is join");

    let mut all_tasks: Vec<JoinHandle<()>> = vec![];
    all_tasks.extend(spot_tasks);
    all_tasks.extend(usdt_tasks);
    all_tasks.push(rpc_task);
    all_tasks.push(check_task);

    for task in all_tasks.into_iter() {
        task.await.unwrap();
    }
}

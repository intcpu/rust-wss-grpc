use crate::rpc::rpc_types::AllBookTickers;
use chrono::Local;
use signal::{
    signal_server::Signal, signal_server::SignalServer, BookTickerReq, BookTickerResp,
    PairBookTickers,
};
use std::collections::HashMap;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info};

pub mod signal {
    tonic::include_proto!("signal");
}

#[derive(Debug, Default)]
pub struct SignalImpl {
    pub all_book_tickers: AllBookTickers, // 添加 pair_bts 字段
}

#[tonic::async_trait]
impl Signal for SignalImpl {
    async fn get_book_tickers(
        &self,
        request: Request<BookTickerReq>,
    ) -> Result<Response<BookTickerResp>, Status> {
        let s_local_time = Local::now();
        let req = request.into_inner();

        let mut all_pair_datas = HashMap::new();
        let usdt_margin_pair_bts = self.all_book_tickers.usdt_margin.data.read().await;
        let spot_pair_bts = self.all_book_tickers.spot.data.read().await;
        for p in req.clone().pairs {
            // pair_datas.insert(p, pair_bts.get(p.as_str()).unwrap().clone())
            let mut pair_datas = PairBookTickers::default();

            if let Some(pair_data) = usdt_margin_pair_bts.get(p.as_str()) {
                pair_datas.tickers.push(pair_data.clone());
                all_pair_datas.insert(p.clone(), pair_datas.clone());
                // println!("{:?}", pair_data);
            } else {
                error!(
                    "RPC SignalServer get_book_tickers usdt_margin_pair not found: {}",
                    p
                );
            }
            if let Some(pair_data) = spot_pair_bts.get(p.as_str()) {
                pair_datas.tickers.push(pair_data.clone());
                all_pair_datas.insert(p.clone(), pair_datas.clone());
                // println!("{:?}", pair_data);
            } else {
                error!(
                    "RPC SignalServer get_book_tickers spot_pair not found: {}",
                    p
                );
            }
        }

        let response = BookTickerResp {
            data: all_pair_datas,
        };

        let e_local_time = Local::now();
        info!(
            "RPC request:{:?} response:{:?} start_time:{:?} end_time:{:?}",
            req,
            response.clone(),
            s_local_time,
            e_local_time
        );
        Ok(Response::new(response))
    }
}

pub async fn rpc_server(all_book_tickers: AllBookTickers) -> Result<(), ()> {
    let addr = "127.0.0.1:50051"
        .parse()
        .expect("RPC Failed to parse address");
    let signal_server = SignalServer::new(SignalImpl { all_book_tickers });

    info!("RPC Server listening on {}", addr);
    match Server::builder()
        .add_service(signal_server)
        .serve(addr)
        .await
    {
        Ok(_) => {
            info!("---RPC Server start ---");
        }
        Err(e) => {
            error!("RPC Server error: {}", e);
            return Err(());
        }
    };
    Ok(())
}

use crate::rpc;
use signal::{signal_server::Signal, signal_server::SignalServer, BookTickerReq, BookTickerResp};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info};

pub mod signal {
    tonic::include_proto!("signal");
}

#[derive(Debug, Default)]
pub struct SignalImpl {
    pub pair_bts: Arc<RwLock<HashMap<String, BookTickerResp>>>, // 添加 pair_bts 字段
}

#[tonic::async_trait]
impl Signal for SignalImpl {
    async fn get_book_tickers(
        &self,
        request: Request<BookTickerReq>,
    ) -> Result<Response<BookTickerResp>, Status> {
        let pair_bts = self.pair_bts.read().await;
        println!("Pair Book Tickers: {:?}", *pair_bts);

        // if let Some(&quantity) = pair_bts.get("banana") {
        //     println!("The quantity of bananas is: {:?}", quantity);
        // } else {
        //     println!("Bananas not found!");
        // }

        let req = request.into_inner();
        println!("Received request: {:?}", req);
        println!("Received request pairs: {:?}", req.pairs);

        // Placeholder logic to process the request and generate a response
        let response = BookTickerResp::default();
        // Populate the response with some data
        // For example:
        // response.set_data(my_data);

        Ok(Response::new(response))
    }
}

pub async fn rpc_server(pair_bts: Arc<RwLock<HashMap<String, BookTickerResp>>>) -> Result<(), ()> {
    let addr = "127.0.0.1:50051".parse().expect("Failed to parse address");
    let signal_server = SignalServer::new(SignalImpl { pair_bts });

    println!("Server listening on {}", addr);
    match Server::builder()
        .add_service(signal_server)
        .serve(addr)
        .await
    {
        Ok(_) => {
            info!("--- Server start ---");
        }
        Err(e) => {
            error!("Server error: {}", e);
            return Err(());
        }
    };
    Ok(())
}

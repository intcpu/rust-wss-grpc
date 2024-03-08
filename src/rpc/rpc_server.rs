use signal::{signal_server::Signal, signal_server::SignalServer, BookTickerReq, BookTickerResp};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info};

pub mod signal {
    tonic::include_proto!("signal");
}

#[derive(Debug, Default)]
pub struct SignalImpl;

#[tonic::async_trait]
impl Signal for SignalImpl {
    async fn get_book_tickers(
        &self,
        request: Request<BookTickerReq>,
    ) -> Result<Response<BookTickerResp>, Status> {
        let req = request.into_inner();
        println!("Received request: {:?}", req);

        // Placeholder logic to process the request and generate a response
        let response = BookTickerResp::default();
        // Populate the response with some data
        // For example:
        // response.set_data(my_data);

        Ok(Response::new(response))
    }
}

pub async fn rpc_server() -> Result<(), ()> {
    let addr = "127.0.0.1:50051".parse().expect("Failed to parse address");
    let signal_server = SignalServer::new(SignalImpl::default());

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

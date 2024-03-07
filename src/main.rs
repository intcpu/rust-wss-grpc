use tracing::Level;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use wss::bookticker;
use wss::wss_test;

mod wss;

// pub mod signal {
//     tonic::include_proto!("signal");
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .finish()
        .try_init()
        .expect("failed to init log");
    let pair = "XRP_USDT";
    println!("------end-----");
    // bookticker::bn_wss_bookticker(pair).await;
    wss_test::wss_test().await.expect("TODO: panic message");
}

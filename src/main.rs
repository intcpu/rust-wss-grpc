use tracing::Level;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use wss::bookticker;
mod wss;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .finish()
        .try_init()
        .expect("failed to init log");
    let pair = "XRP_USDT";
    // let _ = bookticker::gt_wss_bookticker(pair).await;
    println!("------end-----");
    let _ = bookticker::bn_wss_bookticker(pair).await;
}

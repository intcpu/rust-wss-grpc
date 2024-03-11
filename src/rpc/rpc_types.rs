use crate::rpc::rpc_server::signal::PairBookTicker;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

impl Default for AllBookTickers {
    fn default() -> Self {
        AllBookTickers {
            spot: SpotBookTickers {
                data: Arc::new(RwLock::new(HashMap::new())),
            },
            usdt_margin: UsdtMarginBookTickers {
                data: Arc::new(RwLock::new(HashMap::new())),
            },
        }
    }
}

#[derive(Debug)]
pub struct AllBookTickers {
    pub(crate) spot: SpotBookTickers,
    pub(crate) usdt_margin: UsdtMarginBookTickers,
}
#[derive(Debug)]
pub struct SpotBookTickers {
    pub(crate) data: Arc<RwLock<HashMap<String, PairBookTicker>>>,
}
#[derive(Debug)]
pub struct UsdtMarginBookTickers {
    pub(crate) data: Arc<RwLock<HashMap<String, PairBookTicker>>>,
}

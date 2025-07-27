use ethers::providers::{Provider, Http};
use std::sync::Arc;

pub async fn get_provider() -> Arc<Provider<Http>> {
    let rpc_url = std::env::var("POLYGON_RPC").expect("POLYGON_RPC not set");
    Arc::new(Provider::<Http>::try_from(rpc_url).unwrap())
}

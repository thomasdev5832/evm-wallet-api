mod handlers;
mod wallet;
mod provider;

use axum::{Router, routing::get, Server};
use std::net::SocketAddr;
use handlers::{generate_wallet_handler, get_balance_handler};
use std::env;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let rpc = env::var("POLYGON_RPC").expect("POLYGON_RPC not set");
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/wallet", get(generate_wallet_handler))
        .route("/balance/:address", get(get_balance_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

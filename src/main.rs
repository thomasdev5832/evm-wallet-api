mod handlers;
mod wallet;
mod provider;
mod routes;

use axum::{Server};
use std::net::SocketAddr;
use std::env;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let _rpc = env::var("RPC_URL").expect("RPC_URL not set");

    let app = routes::create_routes();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🦀 Server running at http://{}:{} - press CTRL+C to stop", addr.ip(), addr.port());


    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

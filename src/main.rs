mod handlers;
mod wallet;
mod provider;
mod routes;

use axum::{Server};
use std::net::SocketAddr;
use std::env;
use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let _rpc = env::var("RPC_URL").expect("RPC_URL not set");

    let app = routes::create_routes();
    let cors = CorsLayer::new()
        .allow_origin(Any) 
        .allow_methods(Any)
        .allow_headers(Any);

    let app_with_cors = app.layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🦀 Server running at http://{}:{} - press CTRL+C to stop", addr.ip(), addr.port());

    Server::bind(&addr)
        .serve(app_with_cors.into_make_service())
        .await
        .unwrap();
}
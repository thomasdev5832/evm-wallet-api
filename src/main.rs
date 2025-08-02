mod handlers;
mod wallet;
mod provider;
mod routes;

use axum::Server;
use std::net::SocketAddr;
use std::env;
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    // 1. Initialize logging system
    init_tracing();

    // 2. Load environment variables
    dotenv::dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    tracing::info!(rpc_url, "Configuring RPC provider connection");

    // 3. CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 4. HTTP request tracing configuration
    let trace_layer = tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            tracing::info_span!(
                "request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
            tracing::info!("Starting request: {} {}", request.method(), request.uri());
        })
        .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
            tracing::info!(
                "Request completed: status={}, latency={}ms",
                response.status(),
                latency.as_millis()
            );
        })
        .on_failure(
            |error: tower_http::classify::ServerErrorsFailureClass,
             latency: std::time::Duration,
             _span: &tracing::Span| {
                tracing::error!(
                    "Request failed: error={:?}, latency={}ms",
                    error,
                    latency.as_millis()
                );
            },
        );

    // 5. Application setup
    let app = routes::create_routes()
        .layer(cors)
        .layer(trace_layer);

    // 6. Server initialization
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🦀 Server running at http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_tracing() {
    // Log formatting configuration
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,evm_wallet_api=debug,tower_http=debug"))
        )
        .with_target(false) // Cleaner logs without target
        .with_thread_ids(true) // Show thread IDs for async debugging
        .init();

    tracing::info!("Logging system initialized");
}
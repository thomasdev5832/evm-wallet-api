use axum::{Router, routing::{get, post}};
use crate::handlers::{generate_wallet_handler, get_balance_handler};

pub fn create_routes() -> Router {
    Router::new()
        .route("/create-wallet", post(generate_wallet_handler))
        .route("/balance/:address", get(get_balance_handler))
}
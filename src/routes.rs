use axum::{Router, routing::get};
use crate::handlers::{generate_wallet_handler, get_balance_handler};

pub fn create_routes() -> Router {
    Router::new()
        .route("/generate-wallet", get(generate_wallet_handler))
        .route("/get-balance/:addr", get(get_balance_handler))
}

use axum::{extract::Path, response::{IntoResponse, Response}, Json, http::StatusCode};
use serde_json::json;
use crate::{wallet, provider::get_provider};
use ethers::providers::Middleware;
use ethers::types::Address;

pub async fn generate_wallet_handler() -> impl IntoResponse {
    let wallet_data = wallet::generate_wallet();
    Json(wallet_data)
}

pub async fn get_balance_handler(Path(address): Path<String>) -> Response {
    let provider = get_provider().await;

    let addr = match address.parse::<Address>() {
        Ok(a) => a,
        Err(_) => {
            let body = Json(json!({ "error": "Endereço inválido" }));
            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    match provider.get_balance(addr, None).await {
        Ok(balance) => {
            let eth = ethers::utils::format_units(balance, "ether").unwrap();
            Json(json!({ "balance": eth })).into_response()
        }
        Err(e) => {
            let body = Json(json!({ "error": format!("Erro ao obter saldo: {}", e) }));
            (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
        }
    }
}
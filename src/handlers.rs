use axum::{extract::Path, response::{IntoResponse, Response}, Json, http::StatusCode};
use serde_json::json;
use crate::{wallet, provider::get_provider};
use ethers::providers::Middleware;
use ethers::types::{Address, U256};
use ethers::utils::to_checksum;

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

pub async fn get_wallet_info_handler(Path(address): Path<String>) -> Response {
    let provider = get_provider().await;

    match address.parse::<Address>() {
        Ok(addr) => {
            let checksum = to_checksum(&addr, None);
            let is_checksum_valid = address.trim().eq_ignore_ascii_case(&checksum);

            let balance_result = provider.get_balance(addr, None).await;
            let nonce_result = provider.get_transaction_count(addr, None).await;
            let code_result = provider.get_code(addr, None).await;

            let balance = balance_result.unwrap_or_default();
            let nonce = nonce_result.unwrap_or_default();
            let code = code_result.unwrap_or_default();

            let is_contract = !code.0.is_empty();

            let network = std::env::var("NETWORK_NAME").unwrap_or_else(|_| "unknown".to_string());
            let explorer_base = std::env::var("EXPLORER_URL").unwrap_or_else(|_| "".to_string());
            let explorer_url = format!("{}{}", explorer_base, checksum);

            let response = json!({
                "address": checksum,
                "address_lowercase": format!("{:#x}", addr),
                "address_checksum": checksum,
                "is_checksum_valid": is_checksum_valid,
                "network": network,
                "explorer_url": explorer_url,
                "balance": ethers::utils::format_units(balance, "ether").unwrap_or_else(|_| "0".to_string()),
                "nonce": nonce.as_u64(),
                "is_contract": is_contract,
            });

            Json(response).into_response()
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Endereço inválido" })),
        ).into_response(),
    }
}
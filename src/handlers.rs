use axum::{extract::Path, response::{IntoResponse, Response}, Json, http::StatusCode};
use serde::Deserialize;
use serde_json::json;
use crate::wallet;

#[derive(Deserialize)]
pub struct SendTokenRequest {
    pub from_private_key: String,
    pub to_address: String,
    pub amount: String, // Amount in ETH (e.g., "0.1")
}

// Handler to generate a new wallet
pub async fn generate_wallet_handler() -> impl IntoResponse {
    let wallet_data = wallet::generate_wallet();
    Json(wallet_data)
}

// Handler to get balance of an address
pub async fn get_balance_handler(Path(address): Path<String>) -> Response {
    match wallet::get_balance(&address).await {
        Ok(balance) => {
            Json(json!({ "balance": balance })).into_response()
        }
        Err(error) => {
            let status = if error.contains("Invalid address") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status, body).into_response()
        }
    }
}

// Handler to get wallet info
pub async fn get_wallet_info_handler(Path(address): Path<String>) -> Response {
    match wallet::get_wallet_info(&address).await {
        Ok(wallet_info) => {
            Json(wallet_info).into_response()
        }
        Err(error) => {
            let status = if error.contains("Invalid address") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status, body).into_response()
        }
    }
}

// Handler to send tokens
pub async fn send_tokens_handler(Json(payload): Json<SendTokenRequest>) -> Response {
    match wallet::send_tokens(&payload.from_private_key, &payload.to_address, &payload.amount).await {
        Ok(response) => {
            Json(response).into_response()
        }
        Err(error) => {
            let status = if error.contains("Invalid") || error.contains("Insufficient") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status, body).into_response()
        }
    }
}

// Handler to get transaction status
pub async fn get_transaction_status_handler(Path(tx_hash): Path<String>) -> Response {
    match wallet::get_transaction_status(&tx_hash).await {
        Ok(status) => {
            Json(status).into_response()
        }
        Err(error) => {
            let status_code = if error.contains("Invalid transaction hash") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status_code, body).into_response()
        }
    }
}
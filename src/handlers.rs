use axum::{extract::Path, response::{IntoResponse, Response}, Json, http::StatusCode};
use serde::{Deserialize};
use serde_json::json;
use crate::wallet;
use tracing::{info, error, instrument};

#[derive(Debug, Deserialize)]  // Added Debug derive
pub struct SendTokenRequest {
    pub from_private_key: String,
    pub to_address: String,
    pub amount: String, // Amount in ETH (e.g., "0.1")
}

// Handler to generate a new wallet
#[instrument]
pub async fn generate_wallet_handler() -> impl IntoResponse {
    info!("Generating new wallet");
    let wallet_data = wallet::generate_wallet();
    info!("Wallet generated successfully! Address: {}", wallet_data.address);
    Json(wallet_data)
}

// Handler to get balance of an address
#[instrument]
pub async fn get_balance_handler(Path(address): Path<String>) -> Response {
    match wallet::get_balance(&address).await {
        Ok(balance) => {
            info!("Successfully fetched balance for {}: {}", address, balance);
            Json(json!({ "balance": balance })).into_response()
        }
        Err(error) => {
            let status = if error.contains("Invalid address") {
                error!("Invalid address provided: {}", address);
                StatusCode::BAD_REQUEST
            } else {
                error!("Failed to fetch balance for {}: {}", address, error);
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status, body).into_response()
        }
    }
}

// Handler to get wallet info
#[instrument]
pub async fn get_wallet_info_handler(Path(address): Path<String>) -> Response {
    match wallet::get_wallet_info(&address).await {
        Ok(wallet_info) => {
            info!("Successfully fetched wallet info for {}", address);
            Json(wallet_info).into_response()
        }
        Err(error) => {
            let status = if error.contains("Invalid address") {
                error!("Invalid address provided: {}", address);
                StatusCode::BAD_REQUEST
            } else {
                error!("Failed to fetch wallet info for {}: {}", address, error);
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status, body).into_response()
        }
    }
}

// Handler to send tokens
#[instrument]
pub async fn send_tokens_handler(Json(payload): Json<SendTokenRequest>) -> Response {
    let from_key_truncated = format!("{}...", &payload.from_private_key[..6]);
    info!(
        "Sending tokens - From: {}, To: {}, Amount: {} ETH",
        from_key_truncated,
        payload.to_address,
        payload.amount
    );
    
    match wallet::send_tokens(&payload.from_private_key, &payload.to_address, &payload.amount).await {
        Ok(response) => {
            info!(
                "Tokens sent successfully - TX Hash: {}, From: {}, To: {}, Amount: {} ETH",
                response.transaction_hash,
                from_key_truncated,
                payload.to_address,
                payload.amount
            );
            Json(response).into_response()
        }
        Err(error) => {
            let status = if error.contains("Invalid") || error.contains("Insufficient") {
                error!(
                    "Invalid token transfer request - From: {}, To: {}, Amount: {} ETH, Error: {}",
                    from_key_truncated,
                    payload.to_address,
                    payload.amount,
                    error
                );
                StatusCode::BAD_REQUEST
            } else {
                error!(
                    "Failed to send tokens - From: {}, To: {}, Amount: {} ETH, Error: {}",
                    from_key_truncated,
                    payload.to_address,
                    payload.amount,
                    error
                );
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status, body).into_response()
        }
    }
}

// Handler to get transaction status
#[instrument]
pub async fn get_transaction_status_handler(Path(tx_hash): Path<String>) -> Response {
    match wallet::get_transaction_status(&tx_hash).await {
        Ok(status) => {
            info!("Successfully fetched status for TX {}", tx_hash);
            Json(status).into_response()
        }
        Err(error) => {
            let status_code = if error.contains("Invalid transaction hash") {
                error!("Invalid transaction hash provided: {}", tx_hash);
                StatusCode::BAD_REQUEST
            } else {
                error!("Failed to fetch status for TX {}: {}", tx_hash, error);
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            let body = Json(json!({ "error": error }));
            (status_code, body).into_response()
        }
    }
}

// Handler to get transactions
#[instrument]
pub async fn get_transactions_handler(Path(address): Path<String>) -> impl IntoResponse {
    match wallet::get_transactions(&address).await {
        Ok(txs) => {
            info!("Successfully fetched {} transactions for {}", txs.len(), address);
            Json(json!({ "transactions": txs })).into_response()
        },
        Err(e) => {
            error!("Failed to fetch transactions for {}: {}", address, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))).into_response()
        },
    }
}
use axum::{extract::Path, response::{IntoResponse, Response}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{wallet, provider::get_provider};
use ethers::providers::Middleware;
use ethers::types::{Address, TransactionRequest};
use ethers::utils::{to_checksum, parse_units};
use ethers::signers::{LocalWallet, Signer};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct SendTokenRequest {
    pub from_private_key: String,
    pub to_address: String,
    pub amount: String, // Amount in ETH (e.g., "0.1")
}

#[derive(Serialize)]
pub struct SendTokenResponse {
    pub transaction_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub gas_used: Option<String>,
}

// Handler to generate a new wallet
pub async fn generate_wallet_handler() -> impl IntoResponse {
    let wallet_data = wallet::generate_wallet();
    Json(wallet_data)
}

// Handler to get balance of an address
pub async fn get_balance_handler(Path(address): Path<String>) -> Response {
    let provider = get_provider().await;

    let addr = match address.parse::<Address>() {
        Ok(a) => a,
        Err(_) => {
            let body = Json(json!({ "error": "Invalid address" }));
            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    match provider.get_balance(addr, None).await {
        Ok(balance) => {
            let eth = ethers::utils::format_units(balance, "ether").unwrap();
            Json(json!({ "balance": eth })).into_response()
        }
        Err(e) => {
            let body = Json(json!({ "error": format!("Failed to fetch balance: {}", e) }));
            (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
        }
    }
}

// Handler to get wallet info
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
            Json(json!({ "error": "Invalid address" })),
        ).into_response(),
    }
}

// Handler to send tokens
pub async fn send_tokens_handler(Json(payload): Json<SendTokenRequest>) -> Response {
    let provider = get_provider().await;

    // Parse the sender's private key
    let wallet = match LocalWallet::from_str(&payload.from_private_key) {
        Ok(w) => w,
        Err(_) => {
            let body = Json(json!({ "error": "Invalid private key" }));
            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    // Parse the destination address
    let to_address = match payload.to_address.parse::<Address>() {
        Ok(addr) => addr,
        Err(_) => {
            let body = Json(json!({ "error": "Invalid destination address" }));
            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    // Convert ETH amount to wei
    let amount_wei = match parse_units(&payload.amount, "ether") {
        Ok(amount) => amount.into(),
        Err(_) => {
            let body = Json(json!({ "error": "Invalid amount" }));
            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    let from_address = wallet.address();

    // Check if the wallet has enough balance
    match provider.get_balance(from_address, None).await {
        Ok(balance) => {
            if balance < amount_wei {
                let body = Json(json!({ 
                    "error": "Insufficient balance",
                    "current_balance": ethers::utils::format_units(balance, "ether").unwrap_or_else(|_| "0".to_string()),
                    "requested_amount": payload.amount
                }));
                return (StatusCode::BAD_REQUEST, body).into_response();
            }
        }
        Err(e) => {
            let body = Json(json!({ "error": format!("Failed to check balance: {}", e) }));
            return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
        }
    }

    // Get current gas price
    let gas_price = match provider.get_gas_price().await {
        Ok(price) => price,
        Err(e) => {
            let body = Json(json!({ "error": format!("Failed to fetch gas price: {}", e) }));
            return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
        }
    };

    // Build transaction
    let tx = TransactionRequest::new()
        .to(to_address)
        .value(amount_wei)
        .gas_price(gas_price)
        .gas(21000); // Standard gas for ETH transfer

    // Fetch nonce for sender
    let nonce = match provider.get_transaction_count(from_address, None).await {
        Ok(n) => n,
        Err(e) => {
            let body = Json(json!({ "error": format!("Failed to fetch nonce: {}", e) }));
            return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
        }
    };

    let tx = tx.nonce(nonce);

    // Attach chain ID to wallet
    let wallet_with_chain = wallet.with_chain_id(provider.get_chainid().await.unwrap_or_default().as_u64());
    
    let tx_typed = tx.clone().into();

    // Sign transaction
    match wallet_with_chain.sign_transaction(&tx_typed).await {
        Ok(signature) => {
            let signed_tx = tx.rlp_signed(&signature);

            // Send raw transaction to the network
            match provider.send_raw_transaction(signed_tx).await {
                Ok(pending_tx) => {
                    let tx_hash = format!("{:?}", pending_tx.tx_hash());

                    let response = SendTokenResponse {
                        transaction_hash: tx_hash,
                        from_address: to_checksum(&from_address, None),
                        to_address: to_checksum(&to_address, None),
                        amount: payload.amount,
                        gas_used: None, // Only available after transaction is mined
                    };

                    Json(response).into_response()
                }
                Err(e) => {
                    let body = Json(json!({ "error": format!("Failed to send transaction: {}", e) }));
                    (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
                }
            }
        }
        Err(e) => {
            let body = Json(json!({ "error": format!("Failed to sign transaction: {}", e) }));
            (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
        }
    }
}

pub async fn get_transaction_status_handler(Path(tx_hash): Path<String>) -> Response {
    let provider = get_provider().await;

    let tx_hash = match tx_hash.parse::<ethers::types::H256>() {
        Ok(hash) => hash,
        Err(_) => {
            let body = Json(json!({ "error": "Invalid transaction hash" }));
            return (StatusCode::BAD_REQUEST, body).into_response();
        }
    };

    match provider.get_transaction_receipt(tx_hash).await {
        Ok(Some(receipt)) => {
            let status = if receipt.status.unwrap_or_default().is_zero() {
                "failed"
            } else {
                "success"
            };
            let confirmations = provider.get_block_number().await
                .map(|current| current.saturating_sub(receipt.block_number.unwrap_or_default()))
                .unwrap_or_default();
            
            let response = json!({
                "transaction_hash": format!("{:?}", tx_hash),
                "status": status,
                "block_number": receipt.block_number.map(|n| n.as_u64()),
                "gas_used": receipt.gas_used.map(|g| g.to_string()),
                "confirmations": confirmations.as_u64(),
            });

            Json(response).into_response()
        }
        Ok(None) => {
            let body = Json(json!({ "status": "pending" }));
            (StatusCode::OK, body).into_response()
        }
        Err(e) => {
            let body = Json(json!({ "error": format!("Failed to fetch transaction status: {}", e) }));
            (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
        }
    }
}
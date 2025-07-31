use ethers::signers::{LocalWallet, MnemonicBuilder, Signer};
use ethers::providers::Middleware;
use ethers::types::{Address, TransactionRequest, H256};
use ethers::utils::{to_checksum, parse_units, format_units};
use bip39::{Mnemonic, Language};
use rand::RngCore;
use serde::{Deserialize, Serialize}; 
use ethers::utils::hex;
use std::str::FromStr;
use crate::provider::get_provider;

#[derive(Serialize)]
pub struct WalletInfo {
    pub address: String,
    pub private_key: String,
    pub mnemonic: String,
}

#[derive(Serialize)]
pub struct WalletDetails {
    pub address: String,
    pub address_lowercase: String,
    pub address_checksum: String,
    pub is_checksum_valid: bool,
    pub network: String,
    pub explorer_url: String,
    pub balance: String,
    pub nonce: u64,
    pub is_contract: bool,
}

#[derive(Serialize)]
pub struct SendTokenResponse {
    pub transaction_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub gas_used: Option<String>,
}

#[derive(Serialize)]
pub struct TransactionStatus {
    pub transaction_hash: String,
    pub status: String,
    pub block_number: Option<u64>,
    pub gas_used: Option<String>,
    pub confirmations: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    #[serde(rename = "from")]
    pub from: String,
    #[serde(rename = "to")]
    pub to: String,
    #[serde(rename = "value")]
    pub value: String,
    #[serde(rename = "hash")]
    pub hash: String,
    #[serde(rename = "gas")]
    pub gas: String,
    #[serde(rename = "gasPrice")]
    pub gas_price: String,
    #[serde(rename = "timeStamp")]
    pub timestamp: String,
    #[serde(rename = "input")]
    pub input: String,
    #[serde(rename = "isError")]
    pub is_error: String,
    #[serde(rename = "txreceipt_status")]
    pub receipt_status: String,
}

#[derive(Deserialize, Debug)]
pub struct TxListResponse {
    pub status: String,
    pub message: String,
    pub result: Vec<Transaction>,
}

// Generate a new wallet
// This function creates a new wallet with a mnemonic and returns its details.
// It uses a random entropy source to generate the mnemonic.
pub fn generate_wallet() -> WalletInfo {
    let mut entropy = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut entropy);

    // Generate mnemonic with English wordlist
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .expect("Failed to generate mnemonic");
    let phrase = mnemonic.to_string();

    // Use English as the wordlist for MnemonicBuilder
    let wallet: LocalWallet = MnemonicBuilder::<ethers::signers::coins_bip39::English>::default()
        .phrase(phrase.as_str())
        .build()
        .expect("Failed to build wallet");

    WalletInfo {
        address: format!("0x{}", hex::encode(wallet.address())),
        private_key: hex::encode(wallet.signer().to_bytes()),
        mnemonic: phrase,
    }
}

// Get the balance of an address
pub async fn get_balance(address: &str) -> Result<String, String> {
    let provider = get_provider().await;

    let addr = address.parse::<Address>()
        .map_err(|_| "Invalid address".to_string())?;

    let balance = provider.get_balance(addr, None).await
        .map_err(|e| format!("Failed to fetch balance: {}", e))?;

    format_units(balance, "ether")
        .map_err(|_| "Failed to format balance".to_string())
}

// Get complete wallet information
pub async fn get_wallet_info(address: &str) -> Result<WalletDetails, String> {
    let provider = get_provider().await;

    let addr = address.parse::<Address>()
        .map_err(|_| "Invalid address".to_string())?;

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

    let balance_str = format_units(balance, "ether")
        .unwrap_or_else(|_| "0".to_string());

    Ok(WalletDetails {
        address: checksum.clone(),
        address_lowercase: format!("{:#x}", addr),
        address_checksum: checksum,
        is_checksum_valid,
        network,
        explorer_url,
        balance: balance_str,
        nonce: nonce.as_u64(),
        is_contract,
    })
}

// Send tokens
pub async fn send_tokens(from_private_key: &str, to_address: &str, amount: &str) -> Result<SendTokenResponse, String> {
    let provider = get_provider().await;

    // Parse the sender's private key
    let wallet = LocalWallet::from_str(from_private_key)
        .map_err(|_| "Invalid private key".to_string())?;

    // Parse the destination address
    let to_addr = to_address.parse::<Address>()
        .map_err(|_| "Invalid destination address".to_string())?;

    // Convert ETH amount to wei
    let amount_wei = parse_units(amount, "ether")
        .map_err(|_| "Invalid amount".to_string())?
        .into();

    let from_address = wallet.address();

    // Check if the wallet has enough balance
    let balance = provider.get_balance(from_address, None).await
        .map_err(|e| format!("Failed to check balance: {}", e))?;

    if balance < amount_wei {
        let current_balance = format_units(balance, "ether")
            .unwrap_or_else(|_| "0".to_string());
        return Err(format!(
            "Insufficient balance. Current: {} ETH, Requested: {} ETH", 
            current_balance, amount
        ));
    }

    // Get current gas price
    let gas_price = provider.get_gas_price().await
        .map_err(|e| format!("Failed to fetch gas price: {}", e))?;

    // Fetch nonce for sender
    let nonce = provider.get_transaction_count(from_address, None).await
        .map_err(|e| format!("Failed to fetch nonce: {}", e))?;

    // Build transaction
    let tx = TransactionRequest::new()
        .to(to_addr)
        .value(amount_wei)
        .gas_price(gas_price)
        .gas(21000) // Standard gas for ETH transfer
        .nonce(nonce);

    // Attach chain ID to wallet
    let chain_id = provider.get_chainid().await.unwrap_or_default().as_u64();
    let wallet_with_chain = wallet.with_chain_id(chain_id);
    
    let tx_typed = tx.clone().into();

    // Sign transaction
    let signature = wallet_with_chain.sign_transaction(&tx_typed).await
        .map_err(|e| format!("Failed to sign transaction: {}", e))?;

    let signed_tx = tx.rlp_signed(&signature);

    // Send raw transaction to the network
    let pending_tx = provider.send_raw_transaction(signed_tx).await
        .map_err(|e| format!("Failed to send transaction: {}", e))?;

    let tx_hash = format!("{:?}", pending_tx.tx_hash());

    Ok(SendTokenResponse {
        transaction_hash: tx_hash,
        from_address: to_checksum(&from_address, None),
        to_address: to_checksum(&to_addr, None),
        amount: amount.to_string(),
        gas_used: None, // Only available after transaction is mined
    })
}

// Get transaction status
pub async fn get_transaction_status(tx_hash: &str) -> Result<TransactionStatus, String> {
    let provider = get_provider().await;

    let hash = tx_hash.parse::<H256>()
        .map_err(|_| "Invalid transaction hash".to_string())?;

    match provider.get_transaction_receipt(hash).await {
        Ok(Some(receipt)) => {
            let status = if receipt.status.unwrap_or_default().is_zero() {
                "failed"
            } else {
                "success"
            };
            
            let confirmations = provider.get_block_number().await
                .map(|current| current.saturating_sub(receipt.block_number.unwrap_or_default()))
                .unwrap_or_default();
            
            Ok(TransactionStatus {
                transaction_hash: format!("{:?}", hash),
                status: status.to_string(),
                block_number: receipt.block_number.map(|n| n.as_u64()),
                gas_used: receipt.gas_used.map(|g| g.to_string()),
                confirmations: confirmations.as_u64(),
            })
        }
        Ok(None) => {
            Ok(TransactionStatus {
                transaction_hash: format!("{:?}", hash),
                status: "pending".to_string(),
                block_number: None,
                gas_used: None,
                confirmations: 0,
            })
        }
        Err(e) => Err(format!("Failed to fetch transaction status: {}", e))
    }
}

pub async fn get_transactions(address: &str) -> Result<Vec<Transaction>, String> {
    let url = format!(
        "https://rootstock-testnet.blockscout.com/api?module=account&action=txlist&address={}&sort=desc",
        address
    );

    let resp = reqwest::get(&url).await
        .map_err(|e| format!("Failed to fetch transactions: {}", e))?;

    let status = resp.status();
    let text = resp.text().await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    if !status.is_success() {
        return Err(format!("HTTP error {}: {}", status, text));
    }

    let parsed: TxListResponse = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse response: {}\nRaw: {}", e, text))?;

    if parsed.status != "1" {
        return Err(parsed.message);
    }

    Ok(parsed.result)
}
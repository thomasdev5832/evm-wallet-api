use ethers::signers::{LocalWallet, MnemonicBuilder, Signer};
use bip39::{Mnemonic, Language};
use rand::RngCore;
use serde::Serialize;
use ethers::utils::hex;

#[derive(Serialize)]
pub struct WalletInfo {
    pub address: String,
    pub private_key: String,
    pub mnemonic: String,
}

pub fn generate_wallet() -> WalletInfo {
    let mut entropy = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut entropy);

    // Generate mnemonic with English wordlist
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .expect("Failed to generate mnemonic");
    let phrase = mnemonic.to_string(); // Use to_string() to get the mnemonic phrase

    // Use English as the wordlist for MnemonicBuilder
    let wallet: LocalWallet = MnemonicBuilder::<ethers::signers::coins_bip39::English>::default()
        .phrase(phrase.as_str()) // Convert String to &str
        .build()
        .expect("Failed to build wallet");

    WalletInfo {
        address: format!("0x{}", hex::encode(wallet.address())), // Format address with 0x prefix
        private_key: hex::encode(wallet.signer().to_bytes()),
        mnemonic: phrase,
    }
}
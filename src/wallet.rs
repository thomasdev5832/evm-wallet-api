use ethers::signers::{LocalWallet, MnemonicBuilder, Signer};
use bip39::{Mnemonic, Language};
use ethers::core::rand::thread_rng;
use serde::Serialize;
use ethers::utils::hex;

#[derive(Serialize)]
pub struct WalletInfo {
    pub address: String,
    pub private_key: String,
    pub mnemonic: String,
}

pub fn generate_wallet() -> WalletInfo {
    let mnemonic = Mnemonic::new(bip39::MnemonicType::Words12, Language::English);
    let phrase = mnemonic.phrase();

    let wallet: LocalWallet = MnemonicBuilder::default()
        .phrase(phrase)
        .build()
        .unwrap();

    WalletInfo {
        address: wallet.address().to_string(),
        private_key: hex::encode(wallet.signer().to_bytes()),
        mnemonic: phrase.to_string(),
    }
}

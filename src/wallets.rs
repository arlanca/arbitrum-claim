use std::str::FromStr;

use ethers::{
    prelude::k256::ecdsa::SigningKey,
    signers::{coins_bip39::English, MnemonicBuilder, Wallet},
};

use crate::WalletError;

pub(crate) type Wallets = Vec<Wallet<SigningKey>>;

fn from_mnemonic(raw: &str) -> Result<Wallet<SigningKey>, WalletError> {
    MnemonicBuilder::<English>::default()
        .phrase(raw)
        .index(0u32)?
        .build()
        .map_err(|err| err.into())
}

fn from_private(raw: &str) -> Result<Wallet<SigningKey>, WalletError> {
    Wallet::from_str(raw).map_err(|err| err.into())
}

pub fn read_secrets_file(path: &str) -> Result<Wallets, WalletError> {
    let content = std::fs::read_to_string(path)?;

    let wallets = content
        .split('\n')
        .filter_map(|line| {
            let line = line.trim();
            let wallet: Result<Wallet<SigningKey>, _> = if line.len() > 64 {
                from_mnemonic(line)
            } else {
                from_private(line)
            };

            wallet.ok()
        })
        .collect();

    Ok(wallets)
}

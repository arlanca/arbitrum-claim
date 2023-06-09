use std::str::FromStr;

use ethers::{
    prelude::k256::ecdsa::SigningKey,
    signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet},
    types::H160,
};

use crate::{types::SignWallets, Receivers, WalletError};

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

pub fn read_secrets_file(
    path: &str,
    default_receiver: H160,
) -> Result<(SignWallets, Receivers), WalletError> {
    let content = std::fs::read_to_string(path)?;
    let mut receivers = Receivers::new();

    let wallets = content
        .split('\n')
        .flat_map(|line| {
            let line = line.trim();
            let splitted = line.split(':').collect::<Vec<&str>>();
            if splitted.is_empty() {
                return Err(WalletError::InvalidPrivateKey(line.to_string()));
            }

            let credentials = splitted[0].trim();

            let wallet: Result<Wallet<SigningKey>, _> = if credentials.len() > 64 {
                from_mnemonic(credentials)
            } else {
                from_private(credentials)
            };

            match wallet {
                Ok(wallet) => {
                    let receiver = if splitted.len() == 2 {
                        splitted[1]
                            .trim()
                            .parse::<H160>()
                            .unwrap_or(default_receiver)
                    } else {
                        default_receiver
                    };

                    receivers.insert(wallet.address(), receiver);

                    Ok(wallet)
                }
                Err(err) => Err(err),
            }
        })
        .collect();

    Ok((wallets, receivers))
}

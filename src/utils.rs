use std::collections::HashMap;

use ethers::{
    prelude::k256::ecdsa::SigningKey,
    providers::Middleware,
    signers::{Signer, Wallet},
    types::U256,
};
use futures::future::join_all;
use log::warn;

use crate::{Balances, TokenDistributor};

pub async fn fetch_balances<T: Middleware + 'static>(
    token_distributor: &TokenDistributor<T>,
    signers: &[Wallet<SigningKey>],
) -> Balances {
    let future = signers.iter().map(|wallet| async {
        let balance = token_distributor.claimable_tokens(wallet.address()).await;
        if balance.is_err() {
            warn!("Не удалось получить баланс кошелька: {}", wallet.address());
            return None;
        }
        Some((wallet.address(), balance.unwrap()))
    });

    let balances: Balances = join_all(future)
        .await
        .into_iter()
        .filter_map(|p| p)
        .collect::<HashMap<_, U256>>();

    balances
}

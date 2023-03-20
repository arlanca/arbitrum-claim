use ethers::{
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, Bytes, U256},
};
use futures::future::join_all;

use crate::{types::KeyStore, UserWallet};

impl<'a, M: Middleware + 'static> KeyStore<M> {
    pub fn wallets(&self) -> Vec<&UserWallet<M>> {
        self.wallets.values().collect()
    }

    pub async fn get_balances(&mut self) -> U256 {
        let futures = self
            .wallets
            .values_mut()
            .map(|wallet| async { wallet.get_balance().await });

        let balances = join_all(futures).await.into_iter();

        balances.fold(0.into(), |acc_balance, current| acc_balance + current)
    }

    pub async fn fetch_nonces(&mut self) {
        let futures = self
            .wallets
            .values_mut()
            .map(|wallet| async { wallet.fetch_nonce().await });

        join_all(futures).await;
    }

    pub async fn sign_transaction(&self, tx: &TypedTransaction) -> Option<Bytes> {
        let signer = match self.wallets.get(tx.from().unwrap()) {
            Some(signer) => signer,
            None => return None,
        };

        Some(signer.sign_transaction(tx).await)
    }
}

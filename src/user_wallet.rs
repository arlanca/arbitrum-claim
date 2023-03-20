use ethers::{
    abi::AbiEncode,
    providers::Middleware,
    signers::Signer,
    types::{transaction::eip2718::TypedTransaction, Bytes, H160, U256},
};
use log::warn;

use crate::{types::UserWallet, TransferCall};

impl<M: Middleware + 'static> UserWallet<M> {
    pub fn address(&self) -> H160 {
        self.inner.address()
    }

    pub async fn sign_transaction(&self, tx: &TypedTransaction) -> Bytes {
        let sig = self.inner.sign_transaction(tx).await.unwrap();

        tx.rlp_signed(&sig)
    }

    pub async fn get_balance(&mut self) -> U256 {
        match self.distributor.claimable_tokens(self.address()).await {
            Ok(balance) => {
                self.balance = balance;

                balance
            }
            Err(err) => {
                warn!(
                    "Не удалось получить баланс на: {}. Ошибка: {}",
                    self.address(),
                    err.to_string()
                );
                U256::zero()
            }
        }
    }

    pub async fn fetch_nonce(&mut self) {
        match self
            .provider
            .get_transaction_count(self.address(), None)
            .await
        {
            Ok(nonce) => {
                self.nonce = nonce;
            }
            Err(err) => {
                warn!(
                    "Не удалось получить nonce на: {}. Ошибка: {}",
                    self.address(),
                    err.to_string()
                );
            }
        }
    }

    pub fn get_nonce(&self) -> U256 {
        self.nonce
    }

    pub fn get_transfer_input(&self) -> Vec<u8> {
        let call = TransferCall {
            to: self.receiver,
            amount: self.balance,
        };

        AbiEncode::encode(call)
    }
}

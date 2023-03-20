use std::{collections::HashMap, sync::Arc};

use ethers::{
    prelude::k256::ecdsa::SigningKey,
    providers::Middleware,
    signers::{Signer, Wallet},
    types::{transaction::eip2718::TypedTransaction, H160, U256},
};

use crate::TokenDistributor;

pub type Transactions = Vec<TypedTransaction>;
pub type Receivers = HashMap<H160, H160>;
pub type SignWallets = Vec<Wallet<SigningKey>>;

#[derive(Debug, Clone)]
pub struct UserWallet<M: Middleware> {
    pub(crate) inner: Wallet<SigningKey>,
    pub receiver: H160,
    pub(crate) distributor: Arc<TokenDistributor<M>>,
    pub(crate) balance: U256,
    pub(crate) nonce: U256,
    pub(crate) provider: Arc<M>,
}

#[derive(Debug, Clone)]
pub struct KeyStore<M: Middleware> {
    pub(crate) wallets: HashMap<H160, UserWallet<M>>,
}

impl<M: Middleware> UserWallet<M> {
    pub fn new(
        sign_wallet: Wallet<SigningKey>,
        provider: Arc<M>,
        receiver: H160,
        distributor: Arc<TokenDistributor<M>>,
    ) -> Self {
        Self {
            inner: sign_wallet,
            receiver,
            distributor,
            provider,
            balance: U256::zero(),
            nonce: U256::zero(),
        }
    }
}

impl<M: Middleware> KeyStore<M> {
    pub fn make_keystore(
        provider: Arc<M>,
        distributor: Arc<TokenDistributor<M>>,
        wallets: SignWallets,
        receivers: Receivers,
    ) -> Self {
        let wallets = HashMap::from_iter(wallets.into_iter().map(|wallet| {
            let receiver = receivers.get(&wallet.address()).unwrap();

            (
                wallet.address(),
                UserWallet::new(wallet, provider.clone(), *receiver, distributor.clone()),
            )
        }));

        KeyStore { wallets }
    }
}

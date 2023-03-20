use std::collections::HashMap;

use ethers::{types::{H160, Bytes, U256}, signers::Wallet, prelude::k256::ecdsa::SigningKey};

pub type Transactions = Vec<Bytes>;
pub type Receivers = HashMap<H160, H160>;
pub type Wallets = Vec<Wallet<SigningKey>>;


#[derive(Debug)]
pub struct TransactionInfo {
    pub wallet: Wallet<SigningKey>,
    pub balance: U256,
    pub receiver: H160,
}
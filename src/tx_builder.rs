use std::{collections::HashMap, sync::Arc};

use ethers::{
    abi::AbiEncode,
    prelude::k256::ecdsa::SigningKey,
    providers::Middleware,
    signers::{Signer, Wallet},
    types::{transaction::eip2718::TypedTransaction, Bytes, TransactionRequest, H160, U256},
};


use crate::{get_nonce_loop, ClaimCall, TransferCall, ARB_ADDRESS, DISTRIBUTOR_ADDRESS};

pub type Transactions = Vec<Bytes>;
pub type Balances = HashMap<H160, U256>;

#[derive(Debug)]
pub struct ClaimParams {
    pub receiver: H160,
    pub gas_bid: U256,
    pub gas_limit: U256,
}

async fn sign_transaction(signer: &Wallet<SigningKey>, tx: TypedTransaction) -> Bytes {
    let sig = signer.sign_transaction(&tx).await.unwrap();

    tx.rlp_signed(&sig)
}

fn get_transfer_input(to: H160, amount: U256) -> Vec<u8> {
    let call = TransferCall { to, amount };

    AbiEncode::encode(call)
}

pub async fn build_transactions<T: Middleware>(
    provider: Arc<T>,
    signers: &[Wallet<SigningKey>],
    balances: &Balances,
    params: &ClaimParams,
) -> Transactions {
    let mut transactions = vec![];

    let claim_input = AbiEncode::encode(ClaimCall);

    for signer in signers {
        let nonce = get_nonce_loop(&provider, signer.address()).await;

        let balance = balances.get(&signer.address());
        if balance.is_none() {
            continue;
        }
        let balance = balance.unwrap();

        let claim_tx_request = TransactionRequest::new()
            .from(signer.address())
            .to(*DISTRIBUTOR_ADDRESS)
            .data(claim_input.clone())
            .gas(params.gas_limit)
            .gas_price(params.gas_bid)
            .nonce(nonce);

        let transfer_tx_request = TransactionRequest::new()
            .from(signer.address())
            .to(*ARB_ADDRESS)
            .data(get_transfer_input(params.receiver, *balance))
            .gas(params.gas_limit)
            .gas_price(params.gas_bid)
            .nonce(nonce + 1);

        let claim_tx = TypedTransaction::Legacy(claim_tx_request);
        let transfer_tx = TypedTransaction::Legacy(transfer_tx_request);

        transactions.push(sign_transaction(signer, claim_tx).await);
        transactions.push(sign_transaction(signer, transfer_tx).await);
    }

    transactions
}

pub fn build_estimate_tx(from: H160) -> TypedTransaction {
    let claim_input = AbiEncode::encode(ClaimCall);

    let claim_tx_request = TransactionRequest::new()
        .from(from)
        .to(*DISTRIBUTOR_ADDRESS)
        .data(claim_input);

    TypedTransaction::Legacy(claim_tx_request)
}

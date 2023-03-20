use std::sync::Arc;

use ethers::{
    abi::AbiEncode,
    prelude::k256::ecdsa::SigningKey,
    providers::Middleware,
    signers::{Signer, Wallet},
    types::{
        transaction::eip2718::TypedTransaction, Bytes, TransactionRequest, H160, U256,
    },
};

use crate::{get_nonce_loop, TransferCall, ARB_ADDRESS, CLAIM_DATA, DISTRIBUTOR_ADDRESS, TransactionInfo, Transactions};

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
    transactions_infos: &Vec<TransactionInfo>,
    chain_id: u64,
    gas_price: U256,
    gas_limit: U256,
) -> Transactions {
    let mut transactions = vec![];

    for transaction_info in transactions_infos {
        let nonce = get_nonce_loop(&provider, transaction_info.wallet.address()).await;

        let claim_tx_request = TransactionRequest::new()
            .from(transaction_info.wallet.address())
            .to(*DISTRIBUTOR_ADDRESS)
            .data(CLAIM_DATA.clone())
            .gas(gas_limit)
            .gas_price(gas_price)
            .nonce(nonce)
            .chain_id(chain_id);

        let transfer_tx_request = TransactionRequest::new()
            .from(transaction_info.wallet.address())
            .to(*ARB_ADDRESS)
            .data(get_transfer_input(
                transaction_info.receiver,
                transaction_info.balance,
            ))
            .gas(gas_limit)
            .gas_price(gas_price)
            .nonce(nonce + 1)
            .chain_id(chain_id);

        let claim_tx = TypedTransaction::Legacy(claim_tx_request);
        let transfer_tx = TypedTransaction::Legacy(transfer_tx_request);

        transactions.push(sign_transaction(&transaction_info.wallet, claim_tx).await);
        transactions.push(sign_transaction(&transaction_info.wallet, transfer_tx).await);
    }

    transactions
}

pub fn build_estimate_tx(from: H160) -> TypedTransaction {
    let claim_tx_request = TransactionRequest::new()
        .from(from)
        .to(*DISTRIBUTOR_ADDRESS)
        .data(CLAIM_DATA.clone());

    TypedTransaction::Legacy(claim_tx_request)
}

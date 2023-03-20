use ethers::{
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, TransactionRequest, U256},
};

use crate::{types::KeyStore, Transactions, ARB_ADDRESS, CLAIM_DATA, DISTRIBUTOR_ADDRESS};

pub async fn build_transactions<M: Middleware + 'static>(
    keystore: &KeyStore<M>,
    chain_id: u64,
    gas_limit: U256,
    gas_bid: U256,
) -> Transactions {
    let mut transactions = vec![];

    for wallet in keystore.wallets() {
        if wallet.balance.eq(&U256::zero()) {
            continue;
        }

        let claim_tx_request = TransactionRequest::new()
            .from(wallet.address())
            .to(*DISTRIBUTOR_ADDRESS)
            .data(CLAIM_DATA.clone())
            .gas(gas_limit)
            .gas_price(gas_bid)
            .nonce(wallet.get_nonce())
            .chain_id(chain_id);

        let transfer_tx_request = TransactionRequest::new()
            .from(wallet.address())
            .to(*ARB_ADDRESS)
            .data(wallet.get_transfer_input())
            .gas(gas_limit)
            .gas_price(gas_bid)
            .nonce(wallet.get_nonce() + 1)
            .chain_id(chain_id);

        let claim_tx = TypedTransaction::Legacy(claim_tx_request);
        let transfer_tx = TypedTransaction::Legacy(transfer_tx_request);

        transactions.push(claim_tx);
        transactions.push(transfer_tx);
    }

    transactions
}

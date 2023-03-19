use std::{sync::Arc, thread::sleep, time::Duration};

use ethers::{
    prelude::k256::ecdsa::SigningKey,
    providers::{Middleware, MiddlewareError},
    signers::{Signer, Wallet},
    types::{transaction::eip2718::TypedTransaction, Address, Bytes, U256},
};
use futures::future::join_all;
use log::{error, info, warn};
use tokio::task::JoinHandle;

use crate::TokenDistributor;

pub async fn get_wallets_with_balances<T: Middleware + 'static>(
    token_distributor: &TokenDistributor<T>,
    wallets: &[Wallet<SigningKey>],
) -> Vec<(Wallet<SigningKey>, U256)> {
    let futures = wallets.iter().map(|wallet| async move {
        let balance: U256 = match token_distributor.claimable_tokens(wallet.address()).await {
            Ok(balance) => balance,
            Err(_) => {
                warn!("Не удалось получить баланс кошелька: {}", wallet.address());
                return None;
            }
        };

        Some((wallet.clone(), balance))
    });

    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
}

pub async fn wait_gas<T: Middleware>(provider: Arc<T>, tx: TypedTransaction) -> U256 {
    loop {
        let gas = provider.estimate_gas(&tx, None).await;

        if let Ok(gas) = gas {
            return gas;
        }

        let err = {
            let gas_err = gas.unwrap_err();

            match gas_err.as_error_response() {
                Some(json_rpc_err) => json_rpc_err.message.clone(),
                None => gas_err.to_string(),
            }
        };

        warn!("Клейм пока не доступен. Ошибка: {}", err);
        sleep(Duration::from_secs_f64(0.1));
    }
}

pub async fn get_nonce_loop<T: Middleware>(provider: &Arc<T>, address: Address) -> U256 {
    loop {
        let nonce = provider.get_transaction_count(address, None).await;

        if let Ok(nonce) = nonce {
            return nonce;
        };

        let err = {
            let nonce_err = nonce.unwrap_err();

            match nonce_err.as_error_response() {
                Some(json_rpc_err) => json_rpc_err.message.clone(),
                None => nonce_err.to_string(),
            }
        };

        warn!("Не удалось получить nonce. Ошибка: {}", err);
        sleep(Duration::from_secs_f64(0.1));
    }
}

async fn send_transaction<T: Middleware + 'static>(provider: Arc<T>, transaction: Bytes) {
    let tx = provider.send_raw_transaction(transaction).await;
    if let Err(err) = tx {
        error!("Не удалось отправить транзакцию: {:?}", err);
        return;
    };

    let tx = tx.unwrap().await;
    if let Err(err) = tx {
        error!("Транзакция не завершилась успехом: {:?}", err);
        return;
    };

    let tx = tx.unwrap();
    if tx.is_none() {
        error!("Транзакция не завершилась успехом");
        return;
    };

    let tx = tx.unwrap();
    info!(
        "Успешно отправил транзакцию от {}. Хэш: {:#x}",
        tx.from, tx.transaction_hash
    );
}

pub async fn send_transactions<T: Middleware + 'static>(
    provider: Arc<T>,
    transactions: Vec<Bytes>,
) -> Vec<JoinHandle<()>> {
    transactions
        .into_iter()
        .map(|tx| {
            let provider = provider.clone();

            tokio::spawn(send_transaction(provider, tx))
        })
        .collect::<Vec<JoinHandle<()>>>()
}

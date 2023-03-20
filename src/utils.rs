use std::{sync::Arc, thread::sleep, time::Duration};

use ethers::{
    providers::{Middleware, MiddlewareError},
    types::{transaction::eip2718::TypedTransaction, Bytes, U256},
};
use log::{error, info, warn};
use tokio::task::JoinHandle;

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
    if let Some(status) = tx.status {
        if status == 1.into() {
            info!("Транзакция от {} завершилась успехом. Хэш: {:#x}", tx.from, tx.transaction_hash);    
            return;
        }
    }
    
    warn!("Транзакция: {:#x} не завершилась успехом", tx.transaction_hash);
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

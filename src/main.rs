use std::{ops::Add, sync::Arc};

use arbitrum_claim::{
    build_estimate_tx, build_transactions, get_wallets_with_balances, read_secrets_file,
    send_transactions, wait_gas, Config, ProviderError, TokenDistributor, TransactionInfo,
    DISTRIBUTOR_ADDRESS,
};
use ethers::{prelude::*, utils::format_units};
use futures::future::join_all;
use log::{error, info, LevelFilter};

#[tokio::main]
async fn main() -> Result<(), ProviderError> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    // Чтение конфига
    let config = match Config::from_file("config.yaml") {
        Ok(config) => config,
        Err(err) => {
            error!("Не удалось прочитать конфиг. Ошибка: {}", err);
            return Ok(());
        }
    };

    // Подключение к RPC
    let provider = match Provider::<Http>::try_from(&config.rpc) {
        Ok(provider) => Arc::new(provider),
        Err(err) => {
            error!("Не удалось подключиться к RPC: {:?}", err);
            return Ok(());
        }
    };

    // Проверка ноды и получение chain_id
    let chain_id = match provider.get_chainid().await {
        Ok(chain_id) => chain_id,
        Err(err) => {
            error!("Не удалось проверить RPC: {:?}", err);
            return Ok(());
        }
    };

    // Получение секретов
    let (wallets, receivers) = match read_secrets_file(&config.secrets_path, config.receiver) {
        Ok((wallets, receivers)) => (wallets, receivers),
        Err(err) => {
            error!("Не удалось прочитать секреты: {:?}", err);
            return Ok(());
        }
    };

    info!("Количество кошельков: {}", wallets.len());

    // Получение кошельков с балансами
    let wallets_with_balances = get_wallets_with_balances(
        &TokenDistributor::new(*DISTRIBUTOR_ADDRESS, provider.clone()),
        &wallets,
    )
    .await;

    // Получение общего баланса
    let total_balance = wallets_with_balances
        .iter()
        .fold(U256::from(0), |total: U256, (_, balance)| {
            total.add(balance)
        });

    info!("Общий баланс: {}", format_units(total_balance, "ether")?);

    // Создание массива из TransactionInfo
    let transactions_infos = wallets_with_balances
        .into_iter()
        .map(|(wallet, balance)| match receivers.get(&wallet.address()) {
            Some(&receiver) => TransactionInfo {
                wallet,
                balance,
                receiver,
            },
            None => TransactionInfo {
                wallet,
                balance,
                receiver: config.receiver,
            },
        })
        .collect::<Vec<_>>();

    // Получение первого элемента из массива TransactionInfo
    let transaction_info = match transactions_infos.first() {
        Some(transaction_info) => transaction_info,
        None => {
            error!("Нет ни одного кошелька для клейма!");
            return Ok(());
        }
    };

    info!("Кошельков с балансом: {}", transactions_infos.len());

    // Получение газ лимита для клейма
    let gas_limit = wait_gas(
        provider.clone(),
        build_estimate_tx(transaction_info.wallet.address()),
    )
    .await;

    // Получение итогового газ лимита
    let gas_limit = match gas_limit.as_u64() > config.gas_limit {
        true => gas_limit,
        false => config.gas_limit.into(),
    };

    // Создание транзакций для отправки
    let txs = build_transactions(
        provider.clone(),
        &transactions_infos,
        chain_id.as_u64(),
        config.gas_bid,
        gas_limit,
    )
    .await;

    // Отправка транзакций
    let threads = send_transactions(provider.clone(), txs).await;

    // Ожидание завершения
    join_all(threads).await;

    info!("Программа завершила работу!");

    Ok(())
}

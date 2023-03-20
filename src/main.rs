use std::sync::Arc;

use arbitrum_claim::{
    build_transactions, read_secrets_file, send_transactions, wait_gas, Config, KeyStore,
    ProviderError, TokenDistributor, DISTRIBUTOR_ADDRESS,
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

    let distributor = Arc::new(TokenDistributor::new(
        *DISTRIBUTOR_ADDRESS,
        provider.clone(),
    ));

    let mut keystore = KeyStore::make_keystore(provider.clone(), distributor, wallets, receivers);

    info!("Получаю балансы...");
    let total_balance = keystore.get_balances().await;

    info!("Общий баланс: {}", format_units(total_balance, "ether")?);

    info!("Получаю nonce...");
    keystore.fetch_nonces().await;

    info!("Собираю транзакции...");
    let mut transactions = build_transactions(
        &keystore,
        chain_id.as_u64(),
        config.gas_limit.into(),
        config.gas_bid,
    )
    .await;

    let estimate_tx = match transactions.first() {
        Some(estimate_tx) => estimate_tx,
        None => {
            error!("Нет ни одного кошелька для клейма!");
            return Ok(());
        }
    };

    let gas_limit = wait_gas(provider.clone(), estimate_tx.clone()).await;

    if gas_limit > config.gas_limit.into() {
        transactions.iter_mut().for_each(|tx| {
            tx.set_gas(gas_limit);
        });
    }

    let futures = transactions
        .iter()
        .map(|tx| async { keystore.sign_transaction(tx).await });

    let raw_transactions = join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Отправка транзакций
    let threads = send_transactions(provider.clone(), raw_transactions).await;

    // Ожидание завершения
    join_all(threads).await;

    info!("Программа завершила работу!");

    Ok(())
}

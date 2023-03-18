use std::{ops::Add, sync::Arc};

use arbitrum_claim::{
    build_transactions, fetch_balances, read_secrets_file, Balances, Config, ProviderError,
    TokenDistributor, DISTRIBUTOR_ADDRESS,
};
use ethers::{prelude::*, utils::format_units};
use log::{error, info, LevelFilter};

#[tokio::main]
async fn main() -> Result<(), ProviderError> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    // Чтение конфига из файла
    let config = {
        let config = Config::from_file("config.yaml");
        if let Err(err) = config {
            error!("Не удалось прочитать конфиг. Ошибка: {}", err);
            return Ok(());
        }

        config?
    };

    // Подключение к RPC
    let provider = {
        let provider = Provider::<Http>::try_from(&config.rpc);
        if let Err(err) = provider {
            error!("Не удалось подключиться к RPC: {:?}", err);
            return Ok(());
        }

        Arc::new(provider?)
    };

    // Первоначальная проверка работоспособности ноды + получение chainId для signer
    let chain_id = {
        let chain_id = provider.get_chainid().await;
        if let Err(err) = chain_id {
            error!("Не удалось проверить RPC: {:?}", err);
            return Ok(());
        }

        chain_id?
    };

    let wallets = {
        let wallets = read_secrets_file(&config.secrets_path);
        if let Err(err) = wallets {
            error!("Не удалось прочитать файл: {:?}", err);
            return Ok(());
        }

        wallets?
    };

    let signers = wallets
        .into_iter()
        .map(|wallet| wallet.with_chain_id(chain_id.as_u64()))
        .collect::<Vec<Wallet<_>>>();

    info!("Всего кошельков: {}", signers.len());

    let token_distrubitor = TokenDistributor::new(DISTRIBUTOR_ADDRESS.clone(), provider.clone());

    // hashmap с балансами всех юзеров для уменьшения запросов после клейма
    let balances: Balances = fetch_balances(&token_distrubitor, &signers).await;

    // аккумулирование балансов
    let acc_balance = balances
        .values()
        .fold(U256::from(0), |acc: U256, balance: &U256| acc.add(balance));

    info!("Тотал баланс: {}", format_units(acc_balance, "ether")?);

    let claim_params = config.claim_params();

    let _transactions =
        build_transactions(provider.clone(), &signers, &balances, &claim_params).await;

    Ok(())
}

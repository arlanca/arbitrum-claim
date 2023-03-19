use ethers::{
    types::{H160, U256},
    utils::parse_units,
};
use serde::{Deserialize, Deserializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("не удалось прочитать файл {0}")]
    IOError(#[from] std::io::Error),
    #[error("deserialization error: {0}")]
    DeserializationError(#[from] serde_yaml::Error),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub rpc: String,
    pub secrets_path: String,
    pub receiver: H160,
    pub gas_limit: u64,
    #[serde(deserialize_with = "from_float")]
    pub gas_bid: U256,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;

        let config: Config = serde_yaml::from_str(&content)?;

        Ok(config)
    }
}

fn from_float<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let gas: f64 = Deserialize::deserialize(deserializer)?;
    let units = parse_units(gas, "gwei").map_err(serde::de::Error::custom)?;

    Ok(U256::from(units))
}

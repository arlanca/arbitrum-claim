use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("invalid credentials")]
    InvalidCredentials(#[from] ethers::signers::WalletError),
    #[error("unknown file")]
    UnknownFile(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("parse url error {0}")]
    ParseUrl(#[from] url::ParseError),
    #[error("invalid credentials")]
    WalletError(#[from] WalletError),
    #[error("failed to process request")]
    JsonRpcError(#[from] ethers::providers::ProviderError),
    #[error("conversion error {0}")]
    ConversionError(#[from] ethers::utils::ConversionError),
    #[error("config read error {0}")]
    ConfigError(#[from] crate::ConfigError),
}

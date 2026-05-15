//! Error enums shared across the SDK.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    MissingEnv(&'static str),
    #[error("invalid value for env var {0}: {1}")]
    InvalidEnv(&'static str, String),
    #[error("plaintext http:// not allowed without HAAP_ALLOW_HTTP_FOR_DEV=true: {0}")]
    HttpInProduction(&'static str),
    #[error("unknown sealer backend: {0}")]
    UnknownSealerBackend(String),
}

#[derive(Debug, Error)]
pub enum SealerError {
    #[error("sealer backend not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("argon2 key derivation failed: {0}")]
    Argon2(String),
    #[error("AEAD encryption failed: {0}")]
    AeadEncrypt(String),
    #[error("AEAD decryption failed: {0}")]
    AeadDecrypt(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("keyring error: {0}")]
    Keyring(String),
    #[error("missing passphrase env var: {0}")]
    MissingPassphrase(String),
    #[error("ciphertext format invalid: {0}")]
    InvalidFormat(&'static str),
    #[error("backend tag mismatch: bundle was sealed with {0}, this sealer is {1}")]
    BackendTagMismatch(String, String),
}

#[derive(Debug, Error)]
pub enum SubstrateReaderError {
    #[error("redis transport: {0}")]
    Redis(String),
    #[error("deserialization error: {0}")]
    Bincode(#[from] bincode::Error),
}

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("sealer error: {0}")]
    Sealer(#[from] SealerError),
    #[error("substrate reader error: {0}")]
    Substrate(#[from] SubstrateReaderError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("other: {0}")]
    Other(String),
}

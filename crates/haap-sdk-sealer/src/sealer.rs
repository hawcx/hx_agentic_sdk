//! `AgentIdentitySealer` trait.

use async_trait::async_trait;
use haap_sdk_types::{RegisteredAgent, SealedBundle, SealerConfig, SealerError};

use crate::{FileSealer, KmsWrappedSealer, OsKeychainSealer};

#[async_trait]
pub trait AgentIdentitySealer: Send + Sync {
    /// A short tag identifying this sealer backend, embedded in
    /// `SealedBundle::backend_tag` so a later unseal can sanity-check
    /// it's being asked to unseal a bundle of the right shape.
    fn backend_tag(&self) -> &'static str;

    async fn seal(&self, agent: &RegisteredAgent) -> Result<SealedBundle, SealerError>;
    async fn unseal(&self, bundle: &SealedBundle) -> Result<RegisteredAgent, SealerError>;
}

/// Build the sealer specified by `SealerConfig`.
pub fn build_sealer(config: &SealerConfig) -> Result<Box<dyn AgentIdentitySealer>, SealerError> {
    match config {
        SealerConfig::File { path, passphrase_env_var } => Ok(Box::new(FileSealer::new(
            path.clone(),
            passphrase_env_var.clone(),
        ))),
        SealerConfig::OsKeychain { service, account } => Ok(Box::new(OsKeychainSealer::new(
            service.clone(),
            account.clone(),
        ))),
        SealerConfig::KmsWrapped { key_id, region } => Ok(Box::new(KmsWrappedSealer::new(
            key_id.clone(),
            region.clone(),
        ))),
    }
}

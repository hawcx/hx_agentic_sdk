//! Decision logic between fresh registration and unsealing.

use haap_sdk_sealer::AgentIdentitySealer;
use haap_sdk_types::{AuthenticatorConfig, RegisteredAgent, SdkError, SealedBundle};
use std::path::Path;

/// If `config.sealed_agent_path` is set and the file exists, unseal it.
/// Otherwise, perform a fresh registration ceremony and (if a sealed path
/// is configured) write the result there.
///
/// Fresh registration is currently stubbed pending wire-up of the
/// `haap_auth::v6_3_registration::perform_agent_registration` call —
/// see Phase 0 findings doc for the actual signature this function will
/// drive in a follow-up phase.
pub async fn register_or_unseal(
    config: &AuthenticatorConfig,
    sealer: &dyn AgentIdentitySealer,
) -> Result<RegisteredAgent, SdkError> {
    if let Some(path) = &config.sealed_agent_path {
        if path.exists() {
            tracing::info!(path = %path.display(), "Authenticator: unsealing existing agent");
            return unseal_from_disk(sealer, path).await;
        }
    }

    tracing::info!("Authenticator: no sealed bundle present; would perform fresh registration");
    Err(SdkError::Other(
        "fresh registration is wired up in a follow-up phase — Phase 6 docs/phase_0_helper_signatures.md describes the actual perform_agent_registration signature this will invoke".to_string(),
    ))
}

async fn unseal_from_disk(
    sealer: &dyn AgentIdentitySealer,
    path: &Path,
) -> Result<RegisteredAgent, SdkError> {
    let bytes = tokio::fs::read(path).await?;
    let bundle: SealedBundle = bincode::deserialize(&bytes)
        .map_err(|e| SdkError::Other(format!("deserialize sealed bundle: {e}")))?;
    let agent = sealer.unseal(&bundle).await?;
    Ok(agent)
}

/// Persist an agent via the configured sealer to `path`.
pub async fn seal_to_disk(
    sealer: &dyn AgentIdentitySealer,
    agent: &RegisteredAgent,
    path: &Path,
) -> Result<(), SdkError> {
    let bundle = sealer.seal(agent).await?;
    let bytes = bincode::serialize(&bundle)
        .map_err(|e| SdkError::Other(format!("serialize sealed bundle: {e}")))?;
    tokio::fs::write(path, bytes).await?;
    Ok(())
}

//! CLI subcommand implementations.

use anyhow::{anyhow, Result};
use haap_sdk_sealer::build_sealer;
use haap_sdk_types::{AuthenticatorConfig, SealedBundle};
use haap_substrate_reader::CustomerSubstrateReader;

pub async fn register(_user_id: &str, _agent_class: &str, _trust: u8) -> Result<()> {
    let _config = AuthenticatorConfig::from_env()?;
    Err(anyhow!(
        "register: full ceremony wired up in Phase 6 follow-up — see docs/phase_0_helper_signatures.md for the perform_agent_registration signature"
    ))
}

pub async fn seal(input: &str, output: &str) -> Result<()> {
    let config = AuthenticatorConfig::from_env()?;
    let _sealer = build_sealer(&config.sealer)?;
    Err(anyhow!(
        "seal {input} -> {output}: wired up alongside register in Phase 6 follow-up"
    ))
}

pub async fn unseal(input: &str) -> Result<()> {
    let config = AuthenticatorConfig::from_env()?;
    let sealer = build_sealer(&config.sealer)?;
    let bytes = tokio::fs::read(input).await?;
    let bundle: SealedBundle = bincode::deserialize(&bytes)
        .map_err(|e| anyhow!("deserialize sealed bundle: {e}"))?;
    let agent = sealer.unseal(&bundle).await?;
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "client_id": hex::encode(agent.client_id),
        "session_id": agent.session_id,
        "agent_instance_id": hex::encode(agent.agent_instance_id),
        "agent_class": agent.agent_class,
        "trust_level": agent.trust_level,
    }))?);
    Ok(())
}

pub async fn run_supervisor() -> Result<()> {
    Err(anyhow!(
        "run-supervisor: wired up in Phase 10 follow-up — requires authenticator/tqs/assembler binaries on $PATH"
    ))
}

pub async fn run_rsv(listen: &str) -> Result<()> {
    Err(anyhow!(
        "run-rsv on {listen}: wired up in Phase 11 follow-up"
    ))
}

pub async fn substrate_fetch(session_id: u64) -> Result<()> {
    let config = AuthenticatorConfig::from_env()?;
    let url = config
        .customer_redis_url
        .ok_or_else(|| anyhow!("HAAP_CUSTOMER_REDIS_URL not set"))?;
    let mut reader = CustomerSubstrateReader::connect(&url).await?;
    let result = reader.fetch_session(session_id).await?;
    match result {
        Some(m) => println!("{m:?}"),
        None => println!("no session found for {session_id}"),
    }
    Ok(())
}

mod _suppress_unused {
    // Keep TrustLevel import lit in serde_json::json! macro path so a future
    // rustc upgrade doesn't dead-code-eliminate; this is also a known-good
    // re-export site for users importing the CLI as a library.
    #[allow(dead_code)]
    pub fn _force() -> haap_sdk_types::TrustLevel {
        haap_sdk_types::TrustLevel::Verified
    }
}

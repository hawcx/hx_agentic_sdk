//! Authenticator binary entrypoint.
//!
//! Reads config from `HAAP_*` env vars (see `AuthenticatorConfig::from_env`),
//! either unseals an existing `RegisteredAgent` from disk or performs a
//! fresh registration ceremony against the AS, then listens on
//! `authenticator.sock` and serves `IpcMessage::GetSessionKeyRequest` to
//! its peer TQS process.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    haap_authenticator::ipc_server::run().await
}

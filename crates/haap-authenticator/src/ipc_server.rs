//! Authenticator IPC server.
//!
//! Accepts connections on `authenticator.sock` from peers with matching
//! UID (the TQS process), serves `IpcMessage::GetSessionKeyRequest`.

use anyhow::Result;
use haap_sdk_ipc::{ipc_socket_path, peer_cred::current_uid, IpcConnection, IpcServer};
use haap_sdk_sealer::build_sealer;
use haap_sdk_types::{AuthenticatorConfig, IpcMessage};

use crate::registration_flow::register_or_unseal;
use crate::state::AuthenticatorState;

pub async fn run() -> Result<()> {
    let config = AuthenticatorConfig::from_env()?;
    let sealer = build_sealer(&config.sealer)?;

    let agent = register_or_unseal(&config, sealer.as_ref()).await?;
    let state = AuthenticatorState::new(agent);

    let socket_path = ipc_socket_path("authenticator.sock")?;
    let server = IpcServer::bind(&socket_path, current_uid()).await?;
    tracing::info!(socket = %socket_path.display(), "Authenticator IPC server bound");

    loop {
        match server.accept().await {
            Ok(conn) => {
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_conn(conn, state).await {
                        tracing::warn!(error = %e, "Authenticator IPC connection error");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "Authenticator accept error");
            }
        }
    }
}

async fn handle_conn(mut conn: IpcConnection, state: AuthenticatorState) -> Result<()> {
    while let Ok(msg) = conn.recv().await {
        match msg {
            IpcMessage::GetSessionKeyRequest => {
                let snap = state.snapshot().await;
                conn.send(&IpcMessage::session_key_response(
                    snap.k_session_bytes(),
                    snap.session_id,
                    snap.agent_instance_id,
                    snap.verifier_secret_bytes(),
                ))
                .await?;
            }
            IpcMessage::Shutdown => break,
            _ => {
                conn.send(&IpcMessage::Error("unsupported message".to_string())).await?;
            }
        }
    }
    Ok(())
}

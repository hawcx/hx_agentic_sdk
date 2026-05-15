//! TQS IPC server entrypoint.
//!
//! On startup: connect to `authenticator.sock`, fetch K_session, then
//! listen on `tqs.sock` for `IpcMessage::GetTokenRequest` from the
//! Assembler. Pre-minting is left as a follow-up phase since the actual
//! token shape (`haap_wire::ParsedToken`) requires several path-dep'd
//! constructor calls that are best wired up in a focused implementation.

use anyhow::{Context, Result};
use haap_sdk_ipc::{ipc_socket_path, peer_cred::current_uid, IpcClient, IpcConnection, IpcServer};
use haap_sdk_types::IpcMessage;
use std::time::Duration;

pub async fn run() -> Result<()> {
    let auth_socket = ipc_socket_path("authenticator.sock")?;

    tracing::info!(
        authenticator = %auth_socket.display(),
        "TQS: connecting to authenticator"
    );

    // Best-effort retry: the Supervisor spawns Authenticator first and
    // we wait briefly for its socket to be ready.
    let mut auth_conn = match IpcClient::connect(&auth_socket).await {
        Ok(c) => c,
        Err(_) => {
            tokio::time::sleep(Duration::from_millis(250)).await;
            IpcClient::connect(&auth_socket)
                .await
                .context("connect to authenticator.sock failed")?
        }
    };

    auth_conn.send(&IpcMessage::GetSessionKeyRequest).await?;
    let response = auth_conn.recv().await?;

    let (_k_session, _session_id, _agent_instance_id) = match response {
        IpcMessage::SessionKeyResponse {
            k_session,
            session_id,
            agent_instance_id,
            ..
        } => (*k_session, session_id, agent_instance_id),
        other => anyhow::bail!("expected SessionKeyResponse, got {other:?}"),
    };

    let tqs_socket = ipc_socket_path("tqs.sock")?;
    let server = IpcServer::bind(&tqs_socket, current_uid()).await?;
    tracing::info!(socket = %tqs_socket.display(), "TQS IPC server bound");

    loop {
        match server.accept().await {
            Ok(conn) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_conn(conn).await {
                        tracing::warn!(error = %e, "TQS IPC connection error");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "TQS accept error");
            }
        }
    }
}

async fn handle_conn(mut conn: IpcConnection) -> Result<()> {
    while let Ok(msg) = conn.recv().await {
        match msg {
            IpcMessage::GetTokenRequest { audience: _ } => {
                conn.send(&IpcMessage::Error(
                    "TQS pre-mint not wired up — Phase 7 follow-up implements mint_token".to_string(),
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

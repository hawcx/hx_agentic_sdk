//! Assembler IPC server.

use anyhow::Result;
use haap_sdk_ipc::{ipc_socket_path, peer_cred::current_uid, IpcConnection, IpcServer};
use haap_sdk_types::IpcMessage;

use crate::state::AssemblerState;

pub async fn run() -> Result<()> {
    let socket = ipc_socket_path("assembler.sock")?;
    let server = IpcServer::bind(&socket, current_uid()).await?;
    tracing::info!(socket = %socket.display(), "Assembler IPC server bound");

    let state = AssemblerState::new();

    loop {
        match server.accept().await {
            Ok(conn) => {
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_conn(conn, state).await {
                        tracing::warn!(error = %e, "Assembler IPC connection error");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "Assembler accept error");
            }
        }
    }
}

async fn handle_conn(mut conn: IpcConnection, _state: AssemblerState) -> Result<()> {
    while let Ok(msg) = conn.recv().await {
        match msg {
            IpcMessage::AssembleRequest { body: _, audience: _ } => {
                conn.send(&IpcMessage::Error(
                    "Assembler full pipeline (encrypt_request + TQS roundtrip) is wired up in Phase 8 follow-up".to_string(),
                ))
                .await?;
            }
            IpcMessage::DecryptResponse { encrypted_response: _ } => {
                conn.send(&IpcMessage::Error(
                    "decrypt_response is wired up in Phase 8 follow-up".to_string(),
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

//! `AgentRuntime`: top-level facade for agent applications.

use crate::error::SupervisorError;
use crate::supervisor::{Supervisor, SupervisorConfig};
use haap_sdk_ipc::{IpcClient, IpcConnection};
use haap_sdk_types::IpcMessage;

pub struct AgentRuntime {
    pub supervisor: Supervisor,
    pub assembler_conn: IpcConnection,
}

impl AgentRuntime {
    pub async fn new(config: SupervisorConfig) -> Result<Self, SupervisorError> {
        let supervisor = Supervisor::launch(config).await?;
        let assembler_conn = IpcClient::connect(&supervisor.socket_paths.assembler).await?;
        Ok(Self {
            supervisor,
            assembler_conn,
        })
    }

    /// High-level API: encrypt+send an MCP request through the Assembler
    /// pipeline; decrypt the response.
    pub async fn send_request(
        &mut self,
        body: Vec<u8>,
        audience: String,
    ) -> Result<Vec<u8>, SupervisorError> {
        self.assembler_conn
            .send(&IpcMessage::AssembleRequest { body, audience })
            .await?;
        let resp = self.assembler_conn.recv().await?;
        match resp {
            IpcMessage::AssembleResponse { token_bytes: _, encrypted_body: _ } => {
                // Network transport to the MCP server and response decryption
                // are wired up in Phase 10 follow-up.
                Err(SupervisorError::Other(
                    "AgentRuntime full network round-trip is wired up in Phase 10 follow-up"
                        .to_string(),
                ))
            }
            IpcMessage::Error(msg) => Err(SupervisorError::Other(msg)),
            other => Err(SupervisorError::Other(format!("unexpected: {other:?}"))),
        }
    }
}

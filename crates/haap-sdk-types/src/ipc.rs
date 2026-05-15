//! Inter-process IPC message envelope.
//!
//! All IPC frames between Authenticator, TQS, Assembler, and Supervisor
//! deserialize into one of these variants. New variants are appended;
//! never reorder existing variants (bincode is positional and the wire
//! format would change).

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    // ── Authenticator → caller ──
    GetSessionKeyRequest,
    SessionKeyResponse {
        k_session: Box<[u8; 32]>,
        session_id: u64,
        agent_instance_id: [u8; 16],
        verifier_secret: Box<[u8; 32]>,
    },

    // ── TQS → caller ──
    GetTokenRequest {
        audience: String,
    },
    GetTokenResponse {
        token_bytes: Vec<u8>,
        /// The 32-byte response_key the Assembler will use to derive
        /// K_req (via `derive_request_key`) and K_resp.
        response_key: Box<[u8; 32]>,
        session_id: u64,
    },

    // ── Assembler → MCP Client ──
    AssembleRequest {
        body: Vec<u8>,
        audience: String,
    },
    AssembleResponse {
        token_bytes: Vec<u8>,
        encrypted_body: Vec<u8>,
    },
    DecryptResponse {
        encrypted_response: Vec<u8>,
    },
    DecryptedResponse {
        plaintext: Vec<u8>,
    },

    // ── Lifecycle ──
    Shutdown,
    Pong,

    // ── Errors ──
    Error(String),
}

impl IpcMessage {
    /// Construct a `SessionKeyResponse` from raw byte arrays.
    pub fn session_key_response(
        k_session: [u8; 32],
        session_id: u64,
        agent_instance_id: [u8; 16],
        verifier_secret: [u8; 32],
    ) -> Self {
        Self::SessionKeyResponse {
            k_session: Box::new(k_session),
            session_id,
            agent_instance_id,
            verifier_secret: Box::new(verifier_secret),
        }
    }
}

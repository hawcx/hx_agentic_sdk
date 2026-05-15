//! `Token` and `TokenBatch` — SDK-facing token records.
//!
//! The actual wire-format token (`ParsedToken`) lives in `haap-wire`.
//! These SDK types pair the wire-format bytes with the per-token
//! `response_key` that the Assembler needs to derive K_req / K_resp.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// One pre-minted access token produced by the TQS.
#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop, Clone)]
pub struct Token {
    /// 22-byte base64url-encoded JTI per CS §7.1 wire format.
    #[zeroize(skip)]
    pub jti: [u8; 22],
    /// Session ID (u64 per wire format).
    #[zeroize(skip)]
    pub session_id: u64,
    /// Unix epoch seconds.
    #[zeroize(skip)]
    pub issued_at: u64,
    /// Unix epoch seconds.
    #[zeroize(skip)]
    pub expires_at: u64,
    /// The encoded wire-format token bytes.
    pub wire_bytes: Vec<u8>,
    /// The 32-byte response_key from which the Assembler derives K_req
    /// (`derive_request_key(response_key, session_id)`) and K_resp.
    pub response_key: [u8; 32],
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("jti", &String::from_utf8_lossy(&self.jti))
            .field("session_id", &self.session_id)
            .field("issued_at", &self.issued_at)
            .field("expires_at", &self.expires_at)
            .field("wire_bytes_len", &self.wire_bytes.len())
            .field("response_key", &"[REDACTED]")
            .finish()
    }
}

/// A batch of pre-minted tokens produced atomically by the TQS.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBatch {
    pub session_id: u64,
    pub tokens: Vec<Token>,
}

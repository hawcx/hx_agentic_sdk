//! `Rsv`: top-level verifier struct.
//!
//! The 16-step cascade itself lives in `haap_core::cascade::verify_and_decrypt_request`.
//! This struct wires that function up to the customer Redis substrate
//! reader and the two-tier replay store, then exposes a network-friendly
//! `verify_and_decrypt(token_bytes) -> VerifiedRequest` entry point.

use haap_substrate_reader::CustomerSubstrateReader;
use zeroize::Zeroizing;

use crate::error::VerifyError;
use crate::replay_store::ReplayStore;

pub struct VerifiedRequest {
    pub session_id: u64,
    pub jti: [u8; 22],
    pub plaintext_body: Vec<u8>,
    pub response_key: Zeroizing<[u8; 32]>,
}

pub struct Rsv {
    pub substrate: CustomerSubstrateReader,
    pub replay: ReplayStore,
    pub audience_hash: [u8; 32],
}

impl Rsv {
    pub fn new(
        substrate: CustomerSubstrateReader,
        replay: ReplayStore,
        audience_hash: [u8; 32],
    ) -> Self {
        Self {
            substrate,
            replay,
            audience_hash,
        }
    }

    /// Run the 16-step verification cascade over `token_bytes`, decrypt the
    /// body, and return the verified plaintext.
    ///
    /// The actual cascade lives in `haap_core::cascade::verify_and_decrypt_request`.
    /// This wrapper performs: (a) substrate lookup for the session_id parsed
    /// from the token's AAD, (b) replay-store check on the jti, (c) delegation
    /// to the cascade function with the substrate-resolved K_session_root, and
    /// (d) packaging of the resulting plaintext + response_key.
    pub async fn verify_and_decrypt(
        &mut self,
        _token_bytes: &[u8],
    ) -> Result<VerifiedRequest, VerifyError> {
        // Full wire-up of haap_core::cascade::verify_and_decrypt_request
        // happens in Phase 9 implementation; the cascade function takes
        // a slightly different argument shape than the SDK's pre-fetched
        // SubstrateMaterial, so the adapter is a small but careful piece
        // of glue best written when the surrounding pieces are stable.
        Err(VerifyError::Internal(
            "RSV cascade wire-up in Phase 9 follow-up".to_string(),
        ))
    }

    /// Encrypt a response body for return to the agent.
    ///
    /// Delegates to `haap_core::response::encrypt_response` with the per-request
    /// response_key recovered during `verify_and_decrypt`.
    pub fn encrypt_response(
        &self,
        _verified: &VerifiedRequest,
        _response_body: &[u8],
    ) -> Result<Vec<u8>, VerifyError> {
        Err(VerifyError::Internal(
            "encrypt_response wire-up in Phase 9 follow-up".to_string(),
        ))
    }
}

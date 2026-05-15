use haap_sdk_types::SubstrateReaderError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("wire framing invalid: {0}")]
    Framing(String),
    #[error("msg_type unsupported: {0}")]
    UnsupportedMsgType(u8),
    #[error("token outside acceptable timestamp window")]
    TimestampOutOfWindow,
    #[error("token already consumed")]
    Replay,
    #[error("AEAD tag verification failed")]
    AeadFailed,
    #[error("Schnorr signature verification failed")]
    SchnorrFailed,
    #[error("per-token key re-derivation mismatch (Tier 2)")]
    TierTwoFailed,
    #[error("session not found in substrate")]
    SessionNotFound,
    #[error("audience mismatch")]
    AudienceMismatch,
    #[error("token has expired")]
    Expired,
    #[error("policy epoch out of acceptable window")]
    PolicyEpochOutOfWindow,
    #[error("substrate reader: {0}")]
    Substrate(#[from] SubstrateReaderError),
    #[error("wire error: {0}")]
    Wire(String),
    #[error("internal: {0}")]
    Internal(String),
}

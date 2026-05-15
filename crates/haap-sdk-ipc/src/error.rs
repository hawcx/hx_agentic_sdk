use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("frame too large: {0} bytes (max {} bytes)", crate::framing::MAX_FRAME)]
    FrameTooLarge(u32),

    #[error("peer credentials check failed: peer uid {peer_uid} != expected uid {expected_uid}")]
    PeerCredMismatch { peer_uid: u32, expected_uid: u32 },

    #[error("peer credentials unsupported on this platform")]
    PeerCredUnsupported,

    #[error("nix error: {0}")]
    Nix(#[from] nix::Error),

    #[error("socket path not absolute or not under expected directory: {0}")]
    InvalidSocketPath(String),
}

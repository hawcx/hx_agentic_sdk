//! Opaque sealed bundle envelope.

use serde::{Deserialize, Serialize};

/// Opaque container for a sealed `RegisteredAgent`.
///
/// Internal layout depends on the sealer backend. Always treat as
/// opaque bytes — only the matching sealer can unseal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedBundle {
    /// Tag identifying the sealer backend used to produce these bytes,
    /// for safety when multiple backends are configured simultaneously.
    pub backend_tag: String,
    /// The opaque payload.
    pub ciphertext: Vec<u8>,
}

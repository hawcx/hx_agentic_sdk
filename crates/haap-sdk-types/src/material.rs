//! `RegisteredAgent` and `SubstrateMaterial` — the protected key
//! material flowing through the SDK.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::trust::TrustLevel;

/// In-memory record of a successfully registered agent.
///
/// Secret fields are zeroized on drop. Use `seal_for_persistence` on a
/// configured `AgentIdentitySealer` to produce a durable `SealedBundle`
/// rather than persisting these bytes directly.
#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct RegisteredAgent {
    #[zeroize(skip)]
    pub client_id: [u8; 16],
    #[zeroize(skip)]
    pub session_id: u64,
    #[zeroize(skip)]
    pub agent_instance_id: [u8; 16],
    pub agent_class: String,
    #[zeroize(skip)]
    pub trust_level: TrustLevel,

    // ── Secret material (zeroized on drop) ──
    pub(crate) k_session: [u8; 32],
    pub(crate) verifier_secret: [u8; 32],
    pub(crate) ik_i_secret: [u8; 32],
    #[zeroize(skip)]
    pub(crate) ik_i_public: [u8; 32],
}

impl RegisteredAgent {
    pub fn new(
        client_id: [u8; 16],
        session_id: u64,
        agent_instance_id: [u8; 16],
        agent_class: String,
        trust_level: TrustLevel,
        k_session: [u8; 32],
        verifier_secret: [u8; 32],
        ik_i_secret: [u8; 32],
        ik_i_public: [u8; 32],
    ) -> Self {
        Self {
            client_id,
            session_id,
            agent_instance_id,
            agent_class,
            trust_level,
            k_session,
            verifier_secret,
            ik_i_secret,
            ik_i_public,
        }
    }

    pub fn k_session_bytes(&self) -> [u8; 32] {
        self.k_session
    }

    pub fn verifier_secret_bytes(&self) -> [u8; 32] {
        self.verifier_secret
    }

    pub fn ik_i_secret_bytes(&self) -> [u8; 32] {
        self.ik_i_secret
    }

    pub fn ik_i_public_bytes(&self) -> [u8; 32] {
        self.ik_i_public
    }
}

impl std::fmt::Debug for RegisteredAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredAgent")
            .field("client_id", &hex::encode(self.client_id))
            .field("session_id", &self.session_id)
            .field("agent_instance_id", &hex::encode(self.agent_instance_id))
            .field("agent_class", &self.agent_class)
            .field("trust_level", &self.trust_level)
            .field("k_session", &"[REDACTED]")
            .field("verifier_secret", &"[REDACTED]")
            .field("ik_i_secret", &"[REDACTED]")
            .field("ik_i_public", &hex::encode(self.ik_i_public))
            .finish()
    }
}

impl Clone for RegisteredAgent {
    fn clone(&self) -> Self {
        Self {
            client_id: self.client_id,
            session_id: self.session_id,
            agent_instance_id: self.agent_instance_id,
            agent_class: self.agent_class.clone(),
            trust_level: self.trust_level,
            k_session: self.k_session,
            verifier_secret: self.verifier_secret,
            ik_i_secret: self.ik_i_secret,
            ik_i_public: self.ik_i_public,
        }
    }
}

/// Substrate-stored material read by RSV at verify time.
///
/// The CAA writes this to customer Redis under `haap:session:{session_id_hex}`.
#[derive(Serialize, Deserialize, Clone)]
pub struct SubstrateMaterial {
    pub session_id: u64,
    pub k_session_root: [u8; 32],
    pub verifier_secret: [u8; 32],
    pub scope: String,
    pub billing_context: String,
    pub current_epoch_id: u64,
    pub aud_hash: [u8; 32],
}

impl std::fmt::Debug for SubstrateMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubstrateMaterial")
            .field("session_id", &self.session_id)
            .field("k_session_root", &"[REDACTED]")
            .field("verifier_secret", &"[REDACTED]")
            .field("scope", &self.scope)
            .field("billing_context", &self.billing_context)
            .field("current_epoch_id", &self.current_epoch_id)
            .field("aud_hash", &hex::encode(self.aud_hash))
            .finish()
    }
}

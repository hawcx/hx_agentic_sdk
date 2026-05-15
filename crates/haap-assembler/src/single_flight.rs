//! Single-flight enforcement. Pipelining is PROHIBITED per spec.

use haap_sdk_types::Token;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

#[derive(Debug, Error)]
pub enum AssemblerError {
    #[error("a request is already in flight; pipelining is prohibited")]
    AlreadyInFlight,
    #[error("no request in flight; cannot decrypt response")]
    NoInFlight,
    #[error("crypto: {0}")]
    Crypto(String),
}

pub struct InFlightState {
    pub jti: [u8; 22],
    pub session_id: u64,
    pub response_key: Zeroizing<[u8; 32]>,
    pub started_at: Instant,
}

pub struct AssembledRequest {
    pub token: Token,
    pub encrypted_body: Vec<u8>,
}

pub struct SingleFlight {
    inner: Mutex<Option<InFlightState>>,
}

impl SingleFlight {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub async fn begin(&self, state: InFlightState) -> Result<(), AssemblerError> {
        let mut guard = self.inner.lock().await;
        if guard.is_some() {
            return Err(AssemblerError::AlreadyInFlight);
        }
        *guard = Some(state);
        Ok(())
    }

    pub async fn complete(&self) -> Result<InFlightState, AssemblerError> {
        let mut guard = self.inner.lock().await;
        guard.take().ok_or(AssemblerError::NoInFlight)
    }
}

impl Default for SingleFlight {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_state() -> InFlightState {
        InFlightState {
            jti: [0u8; 22],
            session_id: 1,
            response_key: Zeroizing::new([1u8; 32]),
            started_at: Instant::now(),
        }
    }

    #[tokio::test]
    async fn pipelining_is_rejected() {
        let sf = SingleFlight::new();
        sf.begin(mk_state()).await.unwrap();
        let r = sf.begin(mk_state()).await;
        assert!(matches!(r, Err(AssemblerError::AlreadyInFlight)));
    }

    #[tokio::test]
    async fn complete_without_begin_errors() {
        let sf = SingleFlight::new();
        let r = sf.complete().await;
        assert!(matches!(r, Err(AssemblerError::NoInFlight)));
    }

    #[tokio::test]
    async fn begin_then_complete_round_trips() {
        let sf = SingleFlight::new();
        sf.begin(mk_state()).await.unwrap();
        let st = sf.complete().await.unwrap();
        assert_eq!(st.session_id, 1);
    }
}

//! TQS in-memory state: K_session, session_id, audience hash.

use std::sync::Arc;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

use crate::queue::TokenQueue;

#[derive(Clone)]
pub struct TqsState {
    pub k_session: Arc<Zeroizing<[u8; 32]>>,
    pub session_id: u64,
    pub aud_hash: [u8; 32],
    pub queue: Arc<Mutex<TokenQueue>>,
}

impl TqsState {
    pub fn new(
        k_session: [u8; 32],
        session_id: u64,
        aud_hash: [u8; 32],
        queue: TokenQueue,
    ) -> Self {
        Self {
            k_session: Arc::new(Zeroizing::new(k_session)),
            session_id,
            aud_hash,
            queue: Arc::new(Mutex::new(queue)),
        }
    }
}

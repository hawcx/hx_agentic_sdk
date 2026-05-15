//! In-memory `AuthenticatorState` held across IPC requests.

use haap_sdk_types::RegisteredAgent;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AuthenticatorState {
    inner: Arc<RwLock<RegisteredAgent>>,
}

impl AuthenticatorState {
    pub fn new(agent: RegisteredAgent) -> Self {
        Self {
            inner: Arc::new(RwLock::new(agent)),
        }
    }

    pub async fn snapshot(&self) -> RegisteredAgent {
        self.inner.read().await.clone()
    }
}

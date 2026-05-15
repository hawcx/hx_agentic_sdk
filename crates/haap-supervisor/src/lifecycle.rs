//! Lifecycle helpers — waiting for child sockets, graceful shutdown.

use crate::error::SupervisorError;
use std::path::Path;
use std::time::{Duration, Instant};

/// Poll until `path` exists or the timeout elapses.
pub async fn wait_for_socket(
    child: &'static str,
    path: &Path,
    timeout: Duration,
) -> Result<(), SupervisorError> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(SupervisorError::SocketTimeout {
        child,
        socket: path.display().to_string(),
    })
}

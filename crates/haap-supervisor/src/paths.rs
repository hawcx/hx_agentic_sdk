//! Supervisor's view of the IPC socket layout.

use haap_sdk_ipc::{ipc_socket_dir, IpcError};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SocketPaths {
    pub authenticator: PathBuf,
    pub tqs: PathBuf,
    pub assembler: PathBuf,
}

impl SocketPaths {
    pub fn default_paths() -> Result<Self, IpcError> {
        let dir = ipc_socket_dir()?;
        Ok(Self {
            authenticator: dir.join("authenticator.sock"),
            tqs: dir.join("tqs.sock"),
            assembler: dir.join("assembler.sock"),
        })
    }
}

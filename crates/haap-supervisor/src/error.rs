use thiserror::Error;

#[derive(Debug, Error)]
pub enum SupervisorError {
    #[error("failed to spawn {child}: {source}")]
    SpawnFailed {
        child: &'static str,
        source: std::io::Error,
    },
    #[error("child {child} exited unexpectedly with status {status}")]
    ChildExited {
        child: &'static str,
        status: String,
    },
    #[error("timed out waiting for {child} socket {socket} to become ready")]
    SocketTimeout {
        child: &'static str,
        socket: String,
    },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("ipc: {0}")]
    Ipc(#[from] haap_sdk_ipc::IpcError),
    #[error("other: {0}")]
    Other(String),
}

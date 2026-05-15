//! `IpcServer`, `IpcClient`, `IpcConnection` — high-level Unix domain
//! socket abstractions with SO_PEERCRED enforcement.

use crate::error::IpcError;
use crate::framing::{read_frame, write_frame};
use crate::peer_cred::{peer_identity, PeerIdentity};
use haap_sdk_types::IpcMessage;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};

pub struct IpcServer {
    listener: UnixListener,
    expected_peer_uid: u32,
    path: PathBuf,
}

impl IpcServer {
    /// Bind a Unix listener at `path` and remember the expected peer UID
    /// for SO_PEERCRED-enforced peer identity. The listener accepts only
    /// peers whose UID matches `expected_peer_uid`.
    pub async fn bind(path: &Path, expected_peer_uid: u32) -> Result<Self, IpcError> {
        // Unlink any stale socket so binding doesn't fail with EADDRINUSE.
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;

        // Restrict access via filesystem permissions as a defense-in-depth
        // alongside SO_PEERCRED. Mode 0600 means only the owning user
        // can open the socket file at all.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(Self {
            listener,
            expected_peer_uid,
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Accept a connection, returning an `IpcConnection` if and only if
    /// the peer's UID matches `expected_peer_uid`. Otherwise the
    /// connection is dropped and `PeerCredMismatch` is returned.
    pub async fn accept(&self) -> Result<IpcConnection, IpcError> {
        let (stream, _addr) = self.listener.accept().await?;
        let peer = peer_identity(&stream)?;
        if peer.uid != self.expected_peer_uid {
            return Err(IpcError::PeerCredMismatch {
                peer_uid: peer.uid,
                expected_uid: self.expected_peer_uid,
            });
        }
        Ok(IpcConnection { stream, peer })
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub struct IpcClient;

impl IpcClient {
    pub async fn connect(path: &Path) -> Result<IpcConnection, IpcError> {
        let stream = UnixStream::connect(path).await?;
        let peer = peer_identity(&stream)?;
        Ok(IpcConnection { stream, peer })
    }
}

pub struct IpcConnection {
    stream: UnixStream,
    peer: PeerIdentity,
}

impl IpcConnection {
    pub fn peer(&self) -> &PeerIdentity {
        &self.peer
    }

    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), IpcError> {
        let payload = bincode::serialize(msg)?;
        write_frame(&mut self.stream, &payload).await
    }

    pub async fn recv(&mut self) -> Result<IpcMessage, IpcError> {
        let bytes = read_frame(&mut self.stream).await?;
        let msg = bincode::deserialize(&bytes)?;
        Ok(msg)
    }

    pub async fn shutdown(mut self) -> Result<(), IpcError> {
        self.stream.shutdown().await?;
        Ok(())
    }
}

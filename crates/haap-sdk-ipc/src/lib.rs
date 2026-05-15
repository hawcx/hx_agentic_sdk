//! Unix domain socket IPC with SO_PEERCRED-enforced peer identity.
//!
//! All inter-process communication between Authenticator, TQS, Assembler,
//! and Supervisor flows through length-prefixed bincode frames on UDS.
//! `IpcServer::bind` rejects connections from peers whose UID doesn't
//! match the expected UID — the OS-enforced isolation that prevents
//! non-Hawcx processes from connecting.

pub mod connection;
pub mod error;
pub mod framing;
pub mod paths;
pub mod peer_cred;

pub use connection::{IpcClient, IpcConnection, IpcServer};
pub use error::IpcError;
pub use paths::{ipc_socket_dir, ipc_socket_path};
pub use peer_cred::{peer_identity, PeerIdentity};

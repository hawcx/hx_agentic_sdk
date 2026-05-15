//! SDK Supervisor: spawns Authenticator + TQS + Assembler as child
//! processes, manages their lifecycle, exposes `AgentRuntime` to the
//! agent application.
//!
//! The Supervisor itself does NO cryptographic operations — it holds
//! zero key material. Each child process holds only the keys relevant
//! to its role (Authenticator: IK_i; TQS: K_session derived; Assembler:
//! per-request K_req / K_resp).

pub mod error;
pub mod lifecycle;
pub mod paths;
pub mod runtime;
pub mod supervisor;

pub use error::SupervisorError;
pub use paths::SocketPaths;
pub use runtime::AgentRuntime;
pub use supervisor::{Supervisor, SupervisorConfig};

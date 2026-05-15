//! Shared types for the HAAP agentic SDK.
//!
//! This crate centralizes the public types that flow between the four
//! customer-deployed processes (Authenticator, TQS, Assembler, Supervisor)
//! and the MCP-server-side RSV. It is intentionally dependency-light so it
//! can be imported everywhere without pulling in transport, IPC, or crypto
//! libraries.

pub mod config;
pub mod errors;
pub mod ipc;
pub mod material;
pub mod sealed;
pub mod token;
pub mod trust;

pub use config::{AuthenticatorConfig, SealerConfig, SocketPaths};
pub use errors::{ConfigError, SdkError, SealerError, SubstrateReaderError};
pub use ipc::IpcMessage;
pub use material::{RegisteredAgent, SubstrateMaterial};
pub use sealed::SealedBundle;
pub use token::{Token, TokenBatch};
pub use trust::TrustLevel;

//! SDK Token Queue Service: pre-mints batches of access tokens.
//!
//! TQS connects to `authenticator.sock` at startup to fetch K_session via
//! `IpcMessage::GetSessionKeyRequest`, then pre-mints tokens (RECOMMENDED
//! batch 10, hard cap 10K) and serves them to its peer Assembler over
//! `tqs.sock`.
//!
//! Token minting is delegated to `haap_core::mint::mint_token` — the SDK
//! does not reimplement HKDF chains, AEAD constructions, or Schnorr
//! signing. This crate is process scaffolding around that library
//! function plus single-flight queue management.

pub mod ipc_server;
pub mod queue;
pub mod state;

pub use queue::TokenQueue;
pub use state::TqsState;

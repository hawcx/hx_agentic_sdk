//! SDK HAAP Verifier (RSV): MCP-server-side verification cascade.
//!
//! The 16-step cascade per CS §9 is implemented in
//! `haap_core::cascade::verify_and_decrypt_request`. This crate provides:
//! - A network-friendly wrapper around the cascade function
//! - The two-tier replay store (in-process LRU + Redis SETNX)
//! - The customer Redis substrate reader integration
//! - Response encryption helpers
//!
//! Target latency per spec: < 400 μs. Achievable because all crypto
//! operations are cheap (Ristretto255 ops + AES-256-GCM + HKDF), no DH
//! math on the hot path.

pub mod error;
pub mod replay_store;
pub mod rsv;

pub use error::VerifyError;
pub use replay_store::{ReplayError, ReplayStore};
pub use rsv::{Rsv, VerifiedRequest};

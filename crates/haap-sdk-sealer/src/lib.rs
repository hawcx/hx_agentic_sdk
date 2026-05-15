//! Sealer plugin model for persisting `RegisteredAgent` material across
//! process restarts.
//!
//! Three implementations ship:
//! - [`FileSealer`]: Argon2id-derived passphrase key + ChaCha20Poly1305.
//! - [`OsKeychainSealer`]: OS-native keychain via keyring-rs v3.
//! - [`KmsWrappedSealer`]: stub returning NotImplemented (post-alpha).

pub mod file_sealer;
pub mod kms_sealer;
pub mod os_keychain_sealer;
pub mod sealer;

pub use file_sealer::FileSealer;
pub use kms_sealer::KmsWrappedSealer;
pub use os_keychain_sealer::OsKeychainSealer;
pub use sealer::{build_sealer, AgentIdentitySealer};

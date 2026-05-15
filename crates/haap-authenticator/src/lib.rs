//! SDK Authenticator: thin process wrapper around hx_labs's
//! `haap_auth::v6_3_registration::perform_agent_registration`.
//!
//! The library half exposes:
//! - [`run_authenticator`] — the top-level process entrypoint
//! - [`AuthenticatorState`] — in-memory state held across IPC requests
//! - [`register_or_unseal`] — decision logic between fresh registration
//!   and unsealing an existing RegisteredAgent from disk.

pub mod ipc_server;
pub mod registration_flow;
pub mod state;

pub use registration_flow::register_or_unseal;
pub use state::AuthenticatorState;

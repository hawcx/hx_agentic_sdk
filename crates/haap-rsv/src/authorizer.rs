//! Authorizer impl for the RSV cascade.
//!
//! Per Phase 0.4 of docs/rsv_adapter_helper_signatures.md, the
//! `RegistrationScopeAuthorizer` design the prompt described cannot
//! be implemented today: the cascade's `Authorizer` trait signature
//! `authorize(scope, operation, resource) -> bool` has no access to
//! `SessionRecord`, and no `registered_scope` field exists in the
//! substrate.
//!
//! For alpha, we ship `PermissiveAuthorizer` that always returns true.
//! This defers ALL scope/operation/resource authorization to the
//! cascade's existing internal checks:
//!
//! - Step 13 enforces `scope_ceiling` (token's claimed scope must be
//!   within the session's policy-set ceiling)
//! - Step 13 enforces confirmation requirements, PoP, intent
//!   verification, HAAPI billing
//! - Step 14 enforces PoP signature
//!
//! Production deployments may swap in a Cedar-backed Authorizer that
//! evaluates operation+resource against organization policy. That's
//! a separate workstream.
//!
//! Registration-scope semantics (compare token's scope to the agent's
//! registration-time scope strictly) becomes feasible once:
//!
//! 1. The substrate schema (RawSessionRecord) carries a
//!    `registered_scope` field (CAA write-path change).
//! 2. The Authorizer trait is extended to receive `&SessionRecord` or
//!    the SDK uses a stateful Authorizer constructed per-request
//!    after substrate lookup.

use haap_core::Authorizer;

/// Permissive Authorizer: always returns true.
///
/// Cascade-internal checks (scope_ceiling at step 13, PoP at step 14,
/// confirmation requirements, etc.) remain active. This Authorizer
/// only short-circuits the operation+resource policy evaluation that
/// belongs to a future Cedar layer.
pub struct PermissiveAuthorizer;

impl Authorizer for PermissiveAuthorizer {
    fn authorize(&self, _scope: &[u8], _operation: &str, _resource: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissive_authorizer_allows_anything() {
        let auth = PermissiveAuthorizer;
        assert!(auth.authorize(b"any:scope", "read", "any:resource"));
        assert!(auth.authorize(b"", "", ""));
        assert!(auth.authorize(b"write", "DELETE", "/admin"));
    }
}

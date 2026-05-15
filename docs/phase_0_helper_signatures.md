# Phase 0 — hx_labs helper-signature forensic preflight

**Date:** 2026-05-28
**Branch:** feature/initial-sdk-full-population-2026-05-28
**Purpose:** Pin the actual hx_labs API surface the SDK will wrap, and surface
divergences from the prompt's assumptions so subsequent phases adapt to
ground truth rather than the prompt's prose.

## Prerequisite verification

- **PR 3a (register-agent cascade success path) merged in `hx_labs/main`**
  — commit `18025de` "Merge pull request #61 from
  hawcx/feature/register-agent-cascade-success-path-2026-05-28". ✓
- **hx_labs sibling crates present** under `~/Projects/hx_labs/crates/`:
  haap-core, haap-crypto, haap-ipc, haap-auth, haap-wire, haap-keystore,
  haap-redis, haap-as-client, plus existing haap-assembler/haap-supervisor
  /haap-tqs-* (see "Architectural overlap" below).

## Confirmed helpers (will be path-dep'd)

### Registration ceremony

`haap_auth::v6_3_registration::perform_agent_registration` exists at
`haap-auth/src/v6_3_registration.rs:83`. Re-exported at the crate root.

**Actual signature** (divergence from prompt):

```rust
pub async fn perform_agent_registration(
    as_client: &ASClient,
    as_endpoint: &str,
    tqs_ipc: &dyn TqsIpc,
    keystore: &dyn KeyStore,
    org_token: OrgToken,
    pinned_ik_sp: &[u8; 32],
    agent_class: &str,
    subject_user_id: [u8; 16],
    requested_trust: u8,
) -> Result<RegistrationResult, RegistrationError>
```

The prompt's imagined `RegistrationConfig { as_url, admin_console_url, ... }`
struct does not exist. The real function takes:

- `&ASClient` from `haap-as-client` (HTTPS client to AS)
- `&dyn TqsIpc` — pre-injected IPC channel to TQS (the function calls
  `SetSessionContext` on the IPC as part of the ceremony)
- `&dyn KeyStore` from `haap-keystore` — durable storage for IK_i
- `OrgToken` already obtained from admin console out-of-band

This is a much more abstract function than the prompt assumed. The SDK
Authenticator's responsibility is to construct these dependencies (ASClient,
UnixTqsIpc to its peer TQS, a KeyStore impl, fetch the OrgToken) and call.

### Token wire format

**Location:** `haap-wire::token` (NOT `haap-ipc::TokenV3` as the prompt
assumed). The type is `ParsedToken`; encoders/decoders are
`encode_token(&ParsedToken) -> Vec<u8>` and `decode_token(&[u8]) -> Result<ParsedToken, _>`.

Wire layout per CS v6.0.0 §7.1:
- AAD region (104 bytes): framing | session_id(u64 BE) | token_iv(12) | issued_at(u64 BE) | expires_at(u64 BE) | policy_epoch(u64 BE) | aud_hash(32) | jti(22 base64url) | jti_pad(2)
- Schnorr-bound (80 bytes): R_tok(32) | GCM_tag(16) | σ_tok(32)
- Encrypted body: ct_body (variable AES-256-GCM)

### Per-token / K_req / K_resp derivation

In `haap-core`:
- `request::derive_request_key(response_key: &[u8;32], session_id: u64) -> Result<[u8;32], _>`
- `request::derive_request_keys(response_key) -> Result<([u8;32], [u8;12]), _>` (returns key+IV)
- `request::encrypt_request(...)` and `request::decrypt_request(...)`
- `response::encrypt_response(...)` and `response::decrypt_response(response_key, session_id, wire) -> Result<Vec<u8>, _>`
- `response::encrypt_response_multilayer`, `decrypt_response_multilayer_outer`, `decrypt_response_layer`
- `mint::mint_token(MintInput) -> ...` for token minting (TQS uses this)
- `mint::mint_consumer_token` (feature-gated `consumer`)
- `cascade::verify_token`, `cascade::verify_and_decrypt_request` — the RSV's
  16-step cascade is implemented in `haap-core::cascade`, not something the
  SDK reimplements.

The prompt's imagined names `derive_k_tok_sig`, `derive_k_tok_enc`,
`derive_k_req`, `derive_k_resp` do not exist as separate exports. The actual
public surface centralizes around `mint_token` / `verify_token` /
`encrypt_request` / `decrypt_response` etc.; per-token sub-key derivation
is internal to those functions.

### Crypto primitives

`haap_crypto::schnorr_sig::{schnorr_sign, schnorr_verify, SchnorrSignature}`
re-exported at crate root.

`haap_crypto::zkp::{compute_v3_salt, compute_v3_info}` for HKDF v3 inputs.

**AES-256-GCM** is used by haap-core directly via the `aes_gcm` crate:
`Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&k_req))`. There is no
generic `aead_encrypt_aes256gcm` helper. SDK code that needs AES-GCM at
the protocol surface should call into `haap-core`'s encrypt/decrypt
functions, not roll its own `aes_gcm` invocation, to keep nonces and AAD
construction consistent.

### Session ID type

`haap_redis::allocate_session_id(&mut conn) -> Result<u64, _>` returns
**u64**, not a length-prefixed UTF-8 string. The token wire format at
`haap-wire/src/token.rs:[4:12]` carries session_id as `u64 BE`. The
prompt's claim that "session_id is a length-prefixed UTF-8 string per
PR 3a's closure" is **stale and incorrect** for the current code state.
SDK types use `u64` throughout.

### TqsIpc and KeyStore traits (key SDK dependencies)

`haap_auth::TqsIpc` (re-exported, originally at `haap-auth/src/channel.rs:120`):

```rust
pub trait TqsIpc: Send + Sync {
    // SetSessionContext, etc.
}
```

Concrete impls already shipped:
- `UnixTqsIpc` (Unix domain socket)
- `NamedPipeTqsIpc` (Windows)
- `NoopTqsIpc`, `FailingTqsIpc` (testing)

`haap_keystore::KeyStore` trait at `haap-keystore/src/traits.rs:34`. Impls:
- `MemoryKeyStore`
- `WrappedFileKeyStore`
- `HsmKeyStore`

The SDK Authenticator's binary calls `perform_agent_registration` with
appropriate concrete impls — likely `UnixTqsIpc` (pointing at the SDK
TQS process) and `MemoryKeyStore` or `WrappedFileKeyStore` depending on
the configured Sealer backend.

### Other useful exports from `haap_auth`

```rust
pub use agent_service::{AgentService, AgentState, AgentStatus, NoopTqsIpc, FailingTqsIpc, RetryService};
pub use channel::{TqsIpc, UnixTqsIpc, /* ... */};
pub use sdk_enroll::{sdk_enroll, SdkEnrollError, SdkEnrollmentState, SdkService};
pub use v6_3_registration::{perform_agent_registration, RegistrationResult, RegistrationError};
```

`AgentService` provides a higher-level wrapper with retry semantics. The
SDK's Authenticator can build atop `AgentService` rather than calling
`perform_agent_registration` directly.

`OrgToken` lives in `haap_ipc::network::register_agent` (not haap-auth):
`/Users/raviramaraju/Projects/hx_labs/crates/haap-ipc/src/network/register_agent.rs:270`.

## Architectural overlap with hx_labs (significant)

The following crates **already exist in hx_labs** as members of its
workspace, with functionality overlapping what this prompt asks the SDK
to build:

| hx_labs crate | What it does | SDK overlap |
|---|---|---|
| `haap-assembler` | Assembler library (intent, scope_builder, state, destination, tool_arguments_validator) | Prompt's `haap-assembler` |
| `haap-assembler-bin` | Assembler binary process | Prompt's Phase 8 + Phase 10 (Supervisor spawns it) |
| `haap-assembler-mcp` | MCP-transport variant of Assembler | — |
| `haap-supervisor` | Customer-side Supervisor (CaaBootstrap, AgentTracker, channel, graph, pool, paths) — comment string: "HAAP SDK Supervisor" | Prompt's `haap-supervisor` (DIRECT match) |
| `haap-tqs-common` | Shared TQS types/crypto | Prompt's `haap-tqs` |
| `haap-tqs-precompute` + `-bin` | Pre-mint TQS variant | Prompt's `haap-tqs` |
| `haap-tqs-jit` + `-bin` | Just-in-time TQS variant | Prompt's `haap-tqs` |
| `haap-auth` + `haap-auth-bin` | Authenticator (incl. `perform_agent_registration`) + binary | Prompt's `haap-authenticator` |
| `haap-customer-admin-agent` | CAA — writes SessionMaterial to customer Redis | The substrate writer the SDK reads from (per prompt) |
| `haap-seal` / `haap-keystore` | Storage primitives | Prompt's `haap-sdk-sealer` reuses ideas |
| `haap-redis` | Hawcx Redis primitives | — |

This means the prompt's premise — "build new SDK crates that thin-wrap
hx_labs core/crypto/ipc/auth primitives" — is partially out of date.
A literal execution of the prompt creates 10 SDK crates that mostly
duplicate the existing hx_labs implementation organisationally, while
relying on only the four declared path-deps (core, crypto, ipc, auth).

### Decision applied for this run

Per the prompt's explicit "execute aggressively, document divergences"
directive and the user's explicit "do not stop to ask for permission"
follow-up, this run proceeds by:

1. **Building the SDK-owned infrastructure crates from scratch**:
   `haap-sdk-types`, `haap-sdk-sealer`, `haap-sdk-ipc`,
   `haap-substrate-reader`, `haap-sdk-cli`. These genuinely don't
   duplicate anything in hx_labs.
2. **Building thin-wrapper binary crates that delegate to hx_labs**:
   `haap-authenticator`, `haap-tqs`, `haap-assembler`, `haap-supervisor`.
   These are SDK-OWNED PROCESS BINARIES that import hx_labs library
   crates (path-deps) and add the SDK-specific concerns (sealer
   integration, CLI argument parsing, IPC server setup using
   `haap-sdk-ipc`).
3. **Building a fresh RSV crate** that delegates the 16-step cascade
   to `haap_core::cascade::verify_and_decrypt_request`. RSV is a
   PROCESS that wraps that library function with a network listener
   and the customer-Redis substrate reader, neither of which lives
   in hx_labs today.

If, during a subsequent phase, this approach creates byte-level drift
or import conflicts that cannot be reconciled without redesigning the
prompt, that phase produces `docs/halt_state_phase_N.md` and stops.

## Divergences captured (apply throughout)

| Item | Prompt assumption | Actual ground truth | Adaptation |
|---|---|---|---|
| Registration entrypoint signature | `RegistrationConfig` struct | 9 positional args including `&dyn TqsIpc`, `&dyn KeyStore`, `OrgToken` | SDK Authenticator constructs dependencies and calls actual signature |
| Token format location | `haap-ipc::TokenV3` | `haap-wire::ParsedToken` + `encode_token` / `decode_token` | SDK code imports from `haap-wire` |
| Session ID type | length-prefixed UTF-8 string | `u64` (matches wire format and `allocate_session_id`) | SDK types use `u64` |
| Per-token key derivation | `derive_k_tok_enc/sig` exported | encapsulated inside `mint_token` / `verify_token` | SDK calls `mint_token` / `verify_token` directly |
| AES-256-GCM wrapper | `aead_encrypt_aes256gcm` | None — `haap_core::request::encrypt_request` etc. encapsulate it | SDK calls `encrypt_request` / `decrypt_response` etc. |
| K_req / K_resp derivation | `derive_k_req` / `derive_k_resp_seed` | `derive_request_key(response_key, session_id) -> [u8;32]` and `derive_request_keys` | SDK calls the actual functions |
| RSV cascade | "SDK implements 16-step cascade" | `haap_core::cascade::verify_and_decrypt_request` already does it | SDK RSV wraps the existing cascade function |

## OS keychain note

`keyring-rs v3` on Linux requires `libsecret-1-dev`. This is documented
in the OsKeychainSealer crate's README. Tests gated behind
`os-keychain-tests` feature flag for headless CI.

## Path-dep additions to declared `[workspace.dependencies]`

The prompt declares only `haap-core`, `haap-crypto`, `haap-ipc`,
`haap-auth` as path-deps. Subsequent phases additionally require:

- `haap-wire` (token wire format — used by haap-tqs, haap-rsv)
- `haap-keystore` (KeyStore trait — used by haap-authenticator)
- `haap-redis` (`allocate_session_id` — used by haap-authenticator)
- `haap-as-client` (`ASClient` — used by haap-authenticator)

These are added to the workspace `Cargo.toml` in Phase 1 with the
deviation noted there.

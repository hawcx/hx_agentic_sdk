//! SDK Assembler: single-flight enforcement + K_req / K_resp handling.
//!
//! Pipelining is PROHIBITED per spec — max one in-flight response_key at
//! any instant. Request body encryption (under K_req) and response body
//! decryption (under K_resp) delegate to `haap_core::request` and
//! `haap_core::response` respectively; the SDK does not reimplement
//! HKDF chains or AEAD construction.

pub mod ipc_server;
pub mod single_flight;
pub mod state;

pub use single_flight::{AssembledRequest, InFlightState, SingleFlight};
pub use state::AssemblerState;

//! Replay enforcement for the RSV cascade.
//!
//! The cascade's `ReplayCheck` trait is synchronous (precheck +
//! consume). This crate ships two impls:
//!
//! - `InMemReplayCheck`: in-process HashSet of consumed jtis.
//!   Useful for unit tests where Redis isn't desired.
//! - `RedisReplayCheck`: sync Redis SETNX via
//!   `redis::Client::get_connection`. Mirrors hx_labs's
//!   `replay_adapter::RedisReplayCheck` pattern so the SDK doesn't
//!   take a dep on haap-server (which has many other concerns).
//!
//! Both use `haap_redis::replay_key_v070` for byte-identical key
//! naming (`hawcx:replay:{session_id}:{jti_hex}`).

use haap_core::ReplayCheck;
use haap_redis::replay_key_v070;
use redis::Commands;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("redis transport: {0}")]
    Redis(String),
}

// ── In-memory impl (tests + dev) ──────────────────────────────────────

pub struct InMemReplayCheck {
    consumed: HashSet<[u8; 16]>,
}

impl InMemReplayCheck {
    pub fn new() -> Self {
        Self {
            consumed: HashSet::new(),
        }
    }
}

impl Default for InMemReplayCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayCheck for InMemReplayCheck {
    fn replay_precheck(&mut self, _session_id: u64, jti: &[u8; 16]) -> Result<bool, String> {
        Ok(self.consumed.contains(jti))
    }

    fn replay_consume(
        &mut self,
        _session_id: u64,
        jti: &[u8; 16],
        _ttl_secs: u64,
    ) -> Result<bool, String> {
        Ok(self.consumed.insert(*jti))
    }
}

// ── Redis-backed impl (production) ────────────────────────────────────

pub struct RedisReplayCheck {
    conn: redis::Connection,
}

impl RedisReplayCheck {
    pub fn new(conn: redis::Connection) -> Self {
        Self { conn }
    }
}

impl ReplayCheck for RedisReplayCheck {
    fn replay_precheck(&mut self, session_id: u64, jti: &[u8; 16]) -> Result<bool, String> {
        let key = replay_key_v070(session_id, jti);
        self.conn
            .exists::<_, bool>(&key)
            .map_err(|e| format!("redis replay precheck: {e}"))
    }

    fn replay_consume(
        &mut self,
        session_id: u64,
        jti: &[u8; 16],
        ttl_secs: u64,
    ) -> Result<bool, String> {
        let key = replay_key_v070(session_id, jti);
        redis::cmd("SET")
            .arg(&key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query(&mut self.conn)
            .map_err(|e| format!("redis replay consume: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_mem_precheck_initially_false() {
        let mut r = InMemReplayCheck::new();
        let jti = [0u8; 16];
        assert!(!r.replay_precheck(1, &jti).unwrap());
    }

    #[test]
    fn in_mem_consume_first_call_returns_true() {
        let mut r = InMemReplayCheck::new();
        let jti = [1u8; 16];
        assert!(r.replay_consume(1, &jti, 60).unwrap());
    }

    #[test]
    fn in_mem_replay_consume_second_call_returns_false() {
        let mut r = InMemReplayCheck::new();
        let jti = [2u8; 16];
        assert!(r.replay_consume(1, &jti, 60).unwrap());
        assert!(!r.replay_consume(1, &jti, 60).unwrap());
    }

    #[test]
    fn in_mem_precheck_after_consume_returns_true() {
        let mut r = InMemReplayCheck::new();
        let jti = [3u8; 16];
        r.replay_consume(1, &jti, 60).unwrap();
        assert!(r.replay_precheck(1, &jti).unwrap());
    }

    #[test]
    fn different_jti_independent() {
        let mut r = InMemReplayCheck::new();
        let jti1 = [4u8; 16];
        let jti2 = [5u8; 16];
        r.replay_consume(1, &jti1, 60).unwrap();
        assert!(!r.replay_precheck(1, &jti2).unwrap());
    }
}

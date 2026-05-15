//! TQS pre-mint queue with batch refill semantics.

use haap_sdk_types::Token;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct TqsConfig {
    /// RECOMMENDED 10 per batch.
    pub batch_size: usize,
    /// Hard cap per spec: 10K tokens in flight.
    pub max_queue: usize,
    /// RECOMMENDED 60s.
    pub token_ttl_secs: u64,
}

impl Default for TqsConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            max_queue: 10_000,
            token_ttl_secs: 60,
        }
    }
}

pub struct TokenQueue {
    pub config: TqsConfig,
    pub tokens: VecDeque<Token>,
}

impl TokenQueue {
    pub fn new(config: TqsConfig) -> Self {
        Self {
            config,
            tokens: VecDeque::with_capacity(config.batch_size),
        }
    }

    pub fn push(&mut self, token: Token) {
        if self.tokens.len() < self.config.max_queue {
            self.tokens.push_back(token);
        }
    }

    pub fn pop(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

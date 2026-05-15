//! Assembler state.

use crate::single_flight::SingleFlight;
use std::sync::Arc;

#[derive(Clone)]
pub struct AssemblerState {
    pub single_flight: Arc<SingleFlight>,
}

impl AssemblerState {
    pub fn new() -> Self {
        Self {
            single_flight: Arc::new(SingleFlight::new()),
        }
    }
}

impl Default for AssemblerState {
    fn default() -> Self {
        Self::new()
    }
}

//! Trust levels per CS §5.4.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TrustLevel {
    Untrusted = 0,
    Provisional = 1,
    Verified = 2,
    Privileged = 3,
}

impl TrustLevel {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Untrusted),
            1 => Some(Self::Provisional),
            2 => Some(Self::Verified),
            3 => Some(Self::Privileged),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

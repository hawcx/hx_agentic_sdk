//! OsKeychainSealer: cross-platform OS keychain storage of a
//! ChaCha20Poly1305 key, with the ciphertext stored alongside.
//!
//! Wire layout (inside `SealedBundle::ciphertext`):
//! ```text
//! [0:12]   nonce (random per seal)
//! [12:..]  ChaCha20Poly1305 ciphertext (incl 16-byte tag)
//! ```
//! AAD: `b"haap-authenticator-os-keychain-v1"`.
//!
//! The 32-byte key lives in the OS keychain at (service, account).
//!
//! Cross-platform note: Linux requires `libsecret-1-dev`.

use async_trait::async_trait;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use haap_sdk_types::{RegisteredAgent, SealedBundle, SealerError};

use crate::sealer::AgentIdentitySealer;

const AAD: &[u8] = b"haap-authenticator-os-keychain-v1";
const BACKEND_TAG: &str = "os-keychain-v1";

pub struct OsKeychainSealer {
    service: String,
    account: String,
}

impl OsKeychainSealer {
    pub fn new(service: String, account: String) -> Self {
        Self { service, account }
    }

    fn entry(&self) -> Result<keyring::Entry, SealerError> {
        keyring::Entry::new(&self.service, &self.account).map_err(|e| SealerError::Keyring(e.to_string()))
    }

    fn load_or_create_key(&self) -> Result<[u8; 32], SealerError> {
        let entry = self.entry()?;
        match entry.get_password() {
            Ok(hex_string) => {
                let bytes = hex::decode(hex_string.trim()).map_err(|e| SealerError::Keyring(e.to_string()))?;
                if bytes.len() != 32 {
                    return Err(SealerError::InvalidFormat("keychain-stored key not 32 bytes"));
                }
                let mut out = [0u8; 32];
                out.copy_from_slice(&bytes);
                Ok(out)
            }
            Err(keyring::Error::NoEntry) => {
                let mut key = [0u8; 32];
                use rand::RngCore;
                rand::rngs::OsRng.fill_bytes(&mut key);
                entry
                    .set_password(&hex::encode(key))
                    .map_err(|e| SealerError::Keyring(e.to_string()))?;
                Ok(key)
            }
            Err(e) => Err(SealerError::Keyring(e.to_string())),
        }
    }

    fn load_key(&self) -> Result<[u8; 32], SealerError> {
        let entry = self.entry()?;
        let hex_string = entry.get_password().map_err(|e| SealerError::Keyring(e.to_string()))?;
        let bytes = hex::decode(hex_string.trim()).map_err(|e| SealerError::Keyring(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(SealerError::InvalidFormat("keychain-stored key not 32 bytes"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

#[async_trait]
impl AgentIdentitySealer for OsKeychainSealer {
    fn backend_tag(&self) -> &'static str {
        BACKEND_TAG
    }

    async fn seal(&self, agent: &RegisteredAgent) -> Result<SealedBundle, SealerError> {
        let key = self.load_or_create_key()?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));

        let nonce_bytes = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = bincode::serialize(agent)?;

        let ct = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: &plaintext,
                    aad: AAD,
                },
            )
            .map_err(|e| SealerError::AeadEncrypt(e.to_string()))?;

        let mut wire = Vec::with_capacity(12 + ct.len());
        wire.extend_from_slice(&nonce_bytes);
        wire.extend_from_slice(&ct);

        Ok(SealedBundle {
            backend_tag: BACKEND_TAG.to_string(),
            ciphertext: wire,
        })
    }

    async fn unseal(&self, bundle: &SealedBundle) -> Result<RegisteredAgent, SealerError> {
        if bundle.backend_tag != BACKEND_TAG {
            return Err(SealerError::BackendTagMismatch(
                bundle.backend_tag.clone(),
                BACKEND_TAG.to_string(),
            ));
        }
        if bundle.ciphertext.len() < 12 + 16 {
            return Err(SealerError::InvalidFormat("ciphertext shorter than nonce+tag"));
        }

        let key = self.load_key()?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));

        let nonce_bytes = &bundle.ciphertext[..12];
        let ct = &bundle.ciphertext[12..];
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ct,
                    aad: AAD,
                },
            )
            .map_err(|e| SealerError::AeadDecrypt(e.to_string()))?;

        let agent: RegisteredAgent = bincode::deserialize(&plaintext)?;
        Ok(agent)
    }
}

#[cfg(all(test, feature = "os-keychain-tests"))]
mod tests {
    use super::*;
    use haap_sdk_types::TrustLevel;

    fn sample_agent() -> RegisteredAgent {
        RegisteredAgent::new(
            [9u8; 16],
            7,
            [8u8; 16],
            "test-class".to_string(),
            TrustLevel::Verified,
            [10u8; 32],
            [11u8; 32],
            [12u8; 32],
            [13u8; 32],
        )
    }

    #[tokio::test]
    async fn os_keychain_round_trip() {
        let sealer = OsKeychainSealer::new(
            "haap-agentic-sdk-test".to_string(),
            format!("test-{}", std::process::id()),
        );
        let bundle = sealer.seal(&sample_agent()).await.unwrap();
        let recovered = sealer.unseal(&bundle).await.unwrap();
        assert_eq!(recovered.session_id, 7);
    }
}

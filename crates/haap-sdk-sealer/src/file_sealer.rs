//! FileSealer: passphrase + Argon2id → ChaCha20Poly1305 over a file.
//!
//! Wire layout (on-disk inside `SealedBundle::ciphertext`):
//! ```text
//! [0:16]   salt (random per seal)
//! [16:28]  nonce (random per seal)
//! [28:..]  ChaCha20Poly1305 ciphertext (includes 16-byte tag)
//! ```
//! AAD: `b"haap-authenticator-file-sealer-v1"`.

use async_trait::async_trait;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use haap_sdk_types::{RegisteredAgent, SealedBundle, SealerError};
use std::path::PathBuf;
use zeroize::Zeroizing;

use crate::sealer::AgentIdentitySealer;

const AAD: &[u8] = b"haap-authenticator-file-sealer-v1";
const BACKEND_TAG: &str = "file-sealer-v1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

pub struct FileSealer {
    path: PathBuf,
    passphrase_env_var: String,
}

impl FileSealer {
    pub fn new(path: PathBuf, passphrase_env_var: String) -> Self {
        Self {
            path,
            passphrase_env_var,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    fn read_passphrase(&self) -> Result<Zeroizing<String>, SealerError> {
        std::env::var(&self.passphrase_env_var)
            .map(Zeroizing::new)
            .map_err(|_| SealerError::MissingPassphrase(self.passphrase_env_var.clone()))
    }

    fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, SealerError> {
        use argon2::{Algorithm, Argon2, Params, Version};
        let params =
            Params::new(64 * 1024, 3, 4, Some(32)).map_err(|e| SealerError::Argon2(e.to_string()))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut out = Zeroizing::new([0u8; 32]);
        argon
            .hash_password_into(passphrase, salt, out.as_mut())
            .map_err(|e| SealerError::Argon2(e.to_string()))?;
        Ok(out)
    }
}

#[async_trait]
impl AgentIdentitySealer for FileSealer {
    fn backend_tag(&self) -> &'static str {
        BACKEND_TAG
    }

    async fn seal(&self, agent: &RegisteredAgent) -> Result<SealedBundle, SealerError> {
        let passphrase = self.read_passphrase()?;

        // Random salt.
        let mut salt = [0u8; SALT_LEN];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut salt);

        let key = Self::derive_key(passphrase.as_bytes(), &salt)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_ref()));

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

        let mut wire = Vec::with_capacity(SALT_LEN + NONCE_LEN + ct.len());
        wire.extend_from_slice(&salt);
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
        if bundle.ciphertext.len() < SALT_LEN + NONCE_LEN + 16 {
            return Err(SealerError::InvalidFormat(
                "ciphertext shorter than salt+nonce+tag prefix",
            ));
        }

        let salt = &bundle.ciphertext[..SALT_LEN];
        let nonce_bytes = &bundle.ciphertext[SALT_LEN..SALT_LEN + NONCE_LEN];
        let ct = &bundle.ciphertext[SALT_LEN + NONCE_LEN..];

        let passphrase = self.read_passphrase()?;
        let key = Self::derive_key(passphrase.as_bytes(), salt)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use haap_sdk_types::TrustLevel;
    use tempfile::TempDir;

    fn sample_agent() -> RegisteredAgent {
        RegisteredAgent::new(
            [1u8; 16],
            42,
            [2u8; 16],
            "test-class".to_string(),
            TrustLevel::Verified,
            [3u8; 32],
            [4u8; 32],
            [5u8; 32],
            [6u8; 32],
        )
    }

    #[tokio::test]
    async fn seal_unseal_round_trip() {
        let dir = TempDir::new().unwrap();
        std::env::set_var("HAAP_TEST_PASSPHRASE", "correct horse battery staple");
        let sealer = FileSealer::new(dir.path().join("sealed.bin"), "HAAP_TEST_PASSPHRASE".into());
        let agent = sample_agent();

        let bundle = sealer.seal(&agent).await.unwrap();
        let recovered = sealer.unseal(&bundle).await.unwrap();

        assert_eq!(agent.client_id, recovered.client_id);
        assert_eq!(agent.session_id, recovered.session_id);
        assert_eq!(agent.k_session_bytes(), recovered.k_session_bytes());
    }

    #[tokio::test]
    async fn tampered_ciphertext_is_rejected() {
        let dir = TempDir::new().unwrap();
        std::env::set_var("HAAP_TEST_PASSPHRASE_2", "another phrase");
        let sealer =
            FileSealer::new(dir.path().join("sealed.bin"), "HAAP_TEST_PASSPHRASE_2".into());
        let agent = sample_agent();

        let mut bundle = sealer.seal(&agent).await.unwrap();
        // Flip a byte in the ciphertext region.
        let last = bundle.ciphertext.len() - 1;
        bundle.ciphertext[last] ^= 0xFF;
        let result = sealer.unseal(&bundle).await;
        assert!(matches!(result, Err(SealerError::AeadDecrypt(_))));
    }

    #[tokio::test]
    async fn wrong_passphrase_is_rejected() {
        let dir = TempDir::new().unwrap();
        std::env::set_var("HAAP_TEST_PASSPHRASE_3", "right phrase");
        let sealer =
            FileSealer::new(dir.path().join("sealed.bin"), "HAAP_TEST_PASSPHRASE_3".into());
        let bundle = sealer.seal(&sample_agent()).await.unwrap();

        // Swap passphrase.
        std::env::set_var("HAAP_TEST_PASSPHRASE_3", "wrong phrase");
        let result = sealer.unseal(&bundle).await;
        assert!(matches!(result, Err(SealerError::AeadDecrypt(_))));
    }
}

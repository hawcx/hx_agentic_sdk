//! `AuthenticatorConfig` and supporting types.

use crate::errors::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SealerConfig {
    File {
        path: PathBuf,
        passphrase_env_var: String,
    },
    OsKeychain {
        service: String,
        account: String,
    },
    KmsWrapped {
        key_id: String,
        region: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketPaths {
    pub authenticator: PathBuf,
    pub tqs: PathBuf,
    pub assembler: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorConfig {
    pub as_url: String,
    pub admin_console_url: String,
    pub pinned_ik_sp: [u8; 32],
    pub sealer: SealerConfig,
    pub allow_http_for_dev: bool,
    pub customer_redis_url: Option<String>,
    pub sealed_agent_path: Option<PathBuf>,
}

impl AuthenticatorConfig {
    /// Read configuration from `HAAP_*` env vars.
    pub fn from_env() -> Result<Self, ConfigError> {
        let as_url = std::env::var("HAAP_AS_URL")
            .map_err(|_| ConfigError::MissingEnv("HAAP_AS_URL"))?;
        let admin_console_url = std::env::var("HAAP_ADMIN_CONSOLE_URL")
            .map_err(|_| ConfigError::MissingEnv("HAAP_ADMIN_CONSOLE_URL"))?;

        let pinned_ik_sp_hex = std::env::var("HAAP_PINNED_IK_SP")
            .map_err(|_| ConfigError::MissingEnv("HAAP_PINNED_IK_SP"))?;
        let pinned_ik_sp_vec = hex::decode(&pinned_ik_sp_hex)
            .map_err(|e| ConfigError::InvalidEnv("HAAP_PINNED_IK_SP", e.to_string()))?;
        if pinned_ik_sp_vec.len() != 32 {
            return Err(ConfigError::InvalidEnv(
                "HAAP_PINNED_IK_SP",
                format!("expected 32 bytes, got {}", pinned_ik_sp_vec.len()),
            ));
        }
        let mut pinned_ik_sp = [0u8; 32];
        pinned_ik_sp.copy_from_slice(&pinned_ik_sp_vec);

        let sealer = parse_sealer_from_env()?;

        let allow_http_for_dev = std::env::var("HAAP_ALLOW_HTTP_FOR_DEV")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(false);

        if !allow_http_for_dev {
            if as_url.starts_with("http://") {
                return Err(ConfigError::HttpInProduction("HAAP_AS_URL"));
            }
            if admin_console_url.starts_with("http://") {
                return Err(ConfigError::HttpInProduction("HAAP_ADMIN_CONSOLE_URL"));
            }
        }

        let customer_redis_url = std::env::var("HAAP_CUSTOMER_REDIS_URL").ok();
        let sealed_agent_path = std::env::var("HAAP_SEALED_AGENT_PATH").ok().map(PathBuf::from);

        Ok(Self {
            as_url,
            admin_console_url,
            pinned_ik_sp,
            sealer,
            allow_http_for_dev,
            customer_redis_url,
            sealed_agent_path,
        })
    }
}

fn parse_sealer_from_env() -> Result<SealerConfig, ConfigError> {
    let backend = std::env::var("HAAP_SEALER_BACKEND")
        .unwrap_or_else(|_| "file".to_string());

    match backend.as_str() {
        "file" => {
            let path = std::env::var("HAAP_SEALER_FILE_PATH")
                .map_err(|_| ConfigError::MissingEnv("HAAP_SEALER_FILE_PATH"))?;
            let passphrase_env_var = std::env::var("HAAP_SEALER_PASSPHRASE_ENV")
                .unwrap_or_else(|_| "HAAP_SEALER_PASSPHRASE".to_string());
            Ok(SealerConfig::File {
                path: PathBuf::from(path),
                passphrase_env_var,
            })
        }
        "os-keychain" => {
            let service = std::env::var("HAAP_SEALER_KEYCHAIN_SERVICE")
                .unwrap_or_else(|_| "haap-agentic-sdk".to_string());
            let account = std::env::var("HAAP_SEALER_KEYCHAIN_ACCOUNT")
                .unwrap_or_else(|_| "default".to_string());
            Ok(SealerConfig::OsKeychain { service, account })
        }
        "kms" => {
            let key_id = std::env::var("HAAP_SEALER_KMS_KEY_ID")
                .map_err(|_| ConfigError::MissingEnv("HAAP_SEALER_KMS_KEY_ID"))?;
            let region = std::env::var("HAAP_SEALER_KMS_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string());
            Ok(SealerConfig::KmsWrapped { key_id, region })
        }
        other => Err(ConfigError::UnknownSealerBackend(other.to_string())),
    }
}

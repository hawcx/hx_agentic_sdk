//! `Supervisor`: spawns Authenticator + TQS + Assembler as child
//! processes and manages their lifecycle.

use crate::error::SupervisorError;
use crate::lifecycle::wait_for_socket;
use crate::paths::SocketPaths;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::{Child, Command};

#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub authenticator_bin: PathBuf,
    pub tqs_bin: PathBuf,
    pub assembler_bin: PathBuf,
    pub env: HashMap<String, String>,
    pub socket_timeout: Duration,
}

impl SupervisorConfig {
    pub fn new(authenticator_bin: PathBuf, tqs_bin: PathBuf, assembler_bin: PathBuf) -> Self {
        Self {
            authenticator_bin,
            tqs_bin,
            assembler_bin,
            env: HashMap::new(),
            socket_timeout: Duration::from_secs(30),
        }
    }
}

pub struct Supervisor {
    pub authenticator: Child,
    pub tqs: Child,
    pub assembler: Child,
    pub socket_paths: SocketPaths,
}

impl Supervisor {
    pub async fn launch(config: SupervisorConfig) -> Result<Self, SupervisorError> {
        let socket_paths = SocketPaths::default_paths()?;

        let mut authenticator = spawn_child(
            "Authenticator",
            &config.authenticator_bin,
            &config.env,
        )?;

        if let Err(e) =
            wait_for_socket("Authenticator", &socket_paths.authenticator, config.socket_timeout)
                .await
        {
            let _ = authenticator.kill().await;
            return Err(e);
        }

        let mut tqs = spawn_child("TQS", &config.tqs_bin, &config.env)?;
        if let Err(e) = wait_for_socket("TQS", &socket_paths.tqs, config.socket_timeout).await {
            let _ = tqs.kill().await;
            let _ = authenticator.kill().await;
            return Err(e);
        }

        let mut assembler = spawn_child("Assembler", &config.assembler_bin, &config.env)?;
        if let Err(e) =
            wait_for_socket("Assembler", &socket_paths.assembler, config.socket_timeout).await
        {
            let _ = assembler.kill().await;
            let _ = tqs.kill().await;
            let _ = authenticator.kill().await;
            return Err(e);
        }

        Ok(Self {
            authenticator,
            tqs,
            assembler,
            socket_paths,
        })
    }

    pub async fn shutdown(mut self) -> Result<(), SupervisorError> {
        // SIGTERM via tokio::process::Child::kill is actually SIGKILL on
        // Unix; for a real graceful shutdown we'd use libc::kill(pid, SIGTERM).
        // For now: kill all three, swallowing per-child errors so a stuck
        // child doesn't prevent cleanup of the others.
        let _ = self.assembler.kill().await;
        let _ = self.tqs.kill().await;
        let _ = self.authenticator.kill().await;
        Ok(())
    }
}

fn spawn_child(
    name: &'static str,
    bin: &PathBuf,
    env: &HashMap<String, String>,
) -> Result<Child, SupervisorError> {
    let mut cmd = Command::new(bin);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.kill_on_drop(true);
    cmd.spawn().map_err(|e| SupervisorError::SpawnFailed {
        child: name,
        source: e,
    })
}

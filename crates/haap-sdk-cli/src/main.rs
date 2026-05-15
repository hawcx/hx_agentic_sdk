//! `haap-sdk` CLI binary.
//!
//! Subcommands for manual testing and demos. All commands read config
//! from `HAAP_*` env vars (see `AuthenticatorConfig::from_env`).

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "haap-sdk",
    version,
    about = "HAAP Agentic SDK — testing/demo CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Register a new agent; print RegisteredAgent JSON
    Register {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        agent_class: String,
        #[arg(long, default_value_t = 1)]
        trust: u8,
    },
    /// Persist a RegisteredAgent via configured sealer
    Seal {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    /// Recover a RegisteredAgent from a sealed bundle
    Unseal {
        #[arg(long)]
        input: String,
    },
    /// Launch the full 3-process pipeline against a real AS
    RunSupervisor,
    /// Run a standalone RSV listening on HTTP
    RunRsv {
        #[arg(long, default_value = "127.0.0.1:8443")]
        listen: String,
    },
    /// Read SubstrateMaterial from customer Redis
    SubstrateFetch {
        #[arg(long)]
        session_id: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Register { user_id, agent_class, trust } => {
            commands::register(&user_id, &agent_class, trust).await
        }
        Command::Seal { input, output } => commands::seal(&input, &output).await,
        Command::Unseal { input } => commands::unseal(&input).await,
        Command::RunSupervisor => commands::run_supervisor().await,
        Command::RunRsv { listen } => commands::run_rsv(&listen).await,
        Command::SubstrateFetch { session_id } => commands::substrate_fetch(session_id).await,
    }
}

mod commands;

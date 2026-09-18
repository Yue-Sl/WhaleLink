use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};
use whalelink_core::{DaemonConfig, EasyTierProcess};

#[derive(Parser)]
#[command(name = "whalelinkd", about = "WhaleLink local data-plane supervisor")]
struct Args {
    #[arg(long, default_value = "whalelink.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ValidateConfig,
    Run,
    Serve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = DaemonConfig::parse(&fs::read_to_string(&args.config)?)?;
    log_event("configuration.validated");
    match args.command {
        Command::ValidateConfig => {
            println!("configuration schema v{} is valid", config.schema_version)
        }
        Command::Run => {
            let mut process = EasyTierProcess::new(config.easytier);
            log_event("easytier.starting");
            process.start().await?;
            println!("EasyTier process started; Ctrl+C stops the supervisor");
            tokio::signal::ctrl_c().await?;
            process.stop().await?;
            log_event("easytier.stopped");
        }
        Command::Serve => {
            #[cfg(windows)]
            {
                log_event("local_ipc.serving");
                whalelinkd::serve_local_pipe(config.easytier).await?;
            }
            #[cfg(not(windows))]
            anyhow::bail!("Named Pipe transport is only available on Windows");
        }
    }
    Ok(())
}

/// Emits a bounded, machine-readable event. Configuration paths, relay URLs,
/// tokens, invite codes and enrollment material must never be added here.
fn log_event(event: &str) {
    eprintln!(
        "{}",
        serde_json::json!({
            "component": "whalelinkd",
            "event": event,
            "schema_version": 1,
        })
    );
}

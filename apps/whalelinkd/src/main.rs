use clap::{Parser, Subcommand};
use std::{env, fs, path::PathBuf, time::Duration};
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
    /// Starts EasyTier directly from runtime parameters without writing the
    /// room secret to a WhaleLink configuration file.
    Connect {
        /// Absolute path to the checksum-verified EasyTier executable.
        #[arg(long)]
        executable: PathBuf,
        #[arg(long)]
        network_name: String,
        /// Name of an environment variable containing the room secret.
        #[arg(long, default_value = "WHALELINK_NETWORK_SECRET")]
        secret_env: String,
        #[arg(long)]
        relay: String,
        /// Optional EasyTier virtual IPv4 address, for example 10.0.0.20/24.
        #[arg(long)]
        ipv4: Option<String>,
        /// Optional local RPC portal used for EasyTier diagnostics. Omit it
        /// for ordinary connections so no management port is bound.
        #[arg(long)]
        rpc_portal: Option<String>,
        #[arg(long)]
        no_listener: bool,
        /// Disables TUN creation for diagnostics only.
        #[arg(long)]
        no_tun: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::ValidateConfig => {
            let config = load_config(&args.config)?;
            log_event("configuration.validated");
            println!("configuration schema v{} is valid", config.schema_version)
        }
        Command::Run => {
            let config = load_config(&args.config)?;
            log_event("configuration.validated");
            let mut process = EasyTierProcess::new(config.easytier);
            log_event("easytier.starting");
            process.start().await?;
            println!("EasyTier process started; Ctrl+C stops the supervisor");
            tokio::signal::ctrl_c().await?;
            process.stop().await?;
            log_event("easytier.stopped");
        }
        Command::Serve => {
            let config = load_config(&args.config)?;
            log_event("configuration.validated");
            #[cfg(windows)]
            {
                log_event("local_ipc.serving");
                whalelinkd::serve_local_pipe(config.easytier).await?;
            }
            #[cfg(not(windows))]
            anyhow::bail!("Named Pipe transport is only available on Windows");
        }
        Command::Connect {
            executable,
            network_name,
            secret_env,
            relay,
            ipv4,
            rpc_portal,
            no_listener,
            no_tun,
        } => {
            connect(
                executable,
                network_name,
                secret_env,
                relay,
                ipv4,
                rpc_portal,
                no_listener,
                no_tun,
            )
            .await?
        }
    }
    Ok(())
}

fn load_config(path: &PathBuf) -> anyhow::Result<DaemonConfig> {
    Ok(DaemonConfig::parse(&fs::read_to_string(path)?)?)
}

async fn connect(
    executable: PathBuf,
    network_name: String,
    secret_env: String,
    relay: String,
    ipv4: Option<String>,
    rpc_portal: Option<String>,
    no_listener: bool,
    no_tun: bool,
) -> anyhow::Result<()> {
    if !executable.is_absolute() {
        anyhow::bail!("--executable must be an absolute path");
    }
    if !executable.is_file() {
        anyhow::bail!(
            "EasyTier executable does not exist: {}",
            executable.display()
        );
    }
    if network_name.trim().is_empty() || relay.trim().is_empty() {
        anyhow::bail!("--network-name and --relay are required");
    }
    let secret = env::var(&secret_env)
        .map_err(|_| anyhow::anyhow!("environment variable {secret_env} is required"))?;
    if secret.trim().is_empty() {
        anyhow::bail!("the room secret must not be empty");
    }

    let mut arguments = vec![
        "--network-name".to_owned(),
        network_name,
        "--network-secret".to_owned(),
        secret,
        "--disable-upnp".to_owned(),
        "true".to_owned(),
        "--console-log-level".to_owned(),
        "warn".to_owned(),
        "-p".to_owned(),
        relay,
    ];
    if let Some(rpc_portal) = rpc_portal {
        arguments.extend(["--rpc-portal".to_owned(), rpc_portal]);
    }
    if let Some(ipv4) = ipv4 {
        arguments.extend(["-i".to_owned(), ipv4]);
    }
    if no_listener {
        arguments.push("--no-listener".to_owned());
    }
    if no_tun {
        arguments.push("--no-tun".to_owned());
    }

    log_event("easytier.connecting");
    let mut process = EasyTierProcess::new(whalelink_core::EasyTierConfig {
        executable,
        arguments,
    });
    process
        .start()
        .await
        .map_err(|error| anyhow::anyhow!("failed to launch EasyTier: {error}"))?;
    eprintln!(
        "{}",
        serde_json::json!({"component":"whalelinkd","event":"easytier.started","schema_version":1})
    );
    println!("EasyTier connection started; Ctrl+C stops WhaleLink");
    loop {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|error| anyhow::anyhow!("failed to wait for Ctrl+C: {error}"))?;
                break
            },
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                match process.observe_exit() {
                    Ok(true) => anyhow::bail!("EasyTier exited unexpectedly"),
                    Ok(false) => {}
                    Err(error) => anyhow::bail!("failed to inspect EasyTier: {error}"),
                }
            }
        }
    }
    process.stop().await?;
    log_event("easytier.stopped");
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

use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};
use whalelink_core::DaemonConfig;

#[derive(Parser)]
#[command(
    name = "whalelink",
    about = "WhaleLink diagnostics and local daemon CLI"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validates a non-sensitive daemon TOML configuration before daemon startup.
    ValidateConfig {
        #[arg(long)]
        config: PathBuf,
    },
    /// Retrieves local daemon state through the current-user Named Pipe.
    Status,
    /// Starts a room already imported into the current user's protected store.
    Start {
        #[arg(long)]
        room_id: String,
    },
    /// Stops the locally managed EasyTier process.
    Stop,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Args::parse().command {
        Command::ValidateConfig { config } => {
            let parsed = DaemonConfig::parse(&fs::read_to_string(config)?)?;
            println!("valid schema_version={}", parsed.schema_version);
        }
        Command::Status => {
            print_response(pipe_request("status.get", serde_json::Value::Null).await?)?
        }
        Command::Start { room_id } => print_response(
            pipe_request("tunnel.start", serde_json::json!({ "room_id": room_id })).await?,
        )?,
        Command::Stop => {
            print_response(pipe_request("tunnel.stop", serde_json::Value::Null).await?)?
        }
    }
    Ok(())
}

fn print_response(response: whalelink_protocol::IpcResponse) -> anyhow::Result<()> {
    if response.result.ok {
        println!("{}", serde_json::to_string(&response.result.data)?);
        Ok(())
    } else {
        let error = response
            .result
            .error
            .expect("failure response contains an error");
        anyhow::bail!("{}: {}", serde_json::to_value(error.code)?, error.message)
    }
}

#[cfg(windows)]
async fn pipe_request(
    method: &str,
    params: serde_json::Value,
) -> anyhow::Result<whalelink_protocol::IpcResponse> {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::windows::named_pipe::ClientOptions,
    };
    use whalelink_protocol::{IpcRequest, IPC_PROTOCOL_VERSION};

    let mut pipe = ClientOptions::new().open(r"\\.\pipe\WhaleLink.v1")?;
    let request = IpcRequest {
        protocol_version: IPC_PROTOCOL_VERSION,
        request_id: uuid::Uuid::new_v4(),
        method: method.into(),
        params,
    };
    pipe.write_all(serde_json::to_string(&request)?.as_bytes())
        .await?;
    pipe.write_all(b"\n").await?;
    pipe.flush().await?;
    let mut buffer = vec![0_u8; 65_536];
    let read = pipe.read(&mut buffer).await?;
    Ok(serde_json::from_slice(&buffer[..read])?)
}

#[cfg(not(windows))]
async fn pipe_request(
    _method: &str,
    _params: serde_json::Value,
) -> anyhow::Result<whalelink_protocol::IpcResponse> {
    anyhow::bail!("local Named Pipe operations are only available on Windows")
}

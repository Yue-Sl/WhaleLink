use uuid::Uuid;
use whalelink_protocol::{
    ApiEnvelope, ApiError, DataPlaneEnrollment, ErrorCode, IpcRequest, IpcResponse,
    IPC_PROTOCOL_VERSION,
};

/// Handles one decoded IPC request. The Windows Named Pipe transport must only
/// pass messages from the current user to this function.
pub fn handle_ipc(request: IpcRequest, easytier_running: bool) -> IpcResponse {
    let request_id = request.request_id;
    if request.protocol_version != IPC_PROTOCOL_VERSION {
        return failure(
            request_id,
            ErrorCode::VersionIncompatible,
            "unsupported IPC protocol version",
        );
    }
    match request.method.as_str() {
        "version" => success(
            request_id,
            serde_json::json!({ "protocol_version": IPC_PROTOCOL_VERSION }),
        ),
        "health" | "status.get" => success(
            request_id,
            serde_json::json!({ "status": if easytier_running { "running" } else { "stopped" } }),
        ),
        "diagnostics.export" => success(request_id, diagnostic_summary(easytier_running)),
        "tunnel.start" | "tunnel.stop" => failure(
            request_id,
            ErrorCode::Forbidden,
            "process control requires an authenticated local transport",
        ),
        _ => failure(request_id, ErrorCode::NotFound, "unknown IPC method"),
    }
}

fn success(request_id: Uuid, data: serde_json::Value) -> IpcResponse {
    IpcResponse {
        protocol_version: IPC_PROTOCOL_VERSION,
        request_id,
        result: ApiEnvelope::success(data),
    }
}

fn failure(request_id: Uuid, code: ErrorCode, message: &str) -> IpcResponse {
    IpcResponse {
        protocol_version: IPC_PROTOCOL_VERSION,
        request_id,
        result: ApiEnvelope::failure(ApiError {
            code,
            message: message.into(),
            request_id,
        }),
    }
}

#[cfg(windows)]
pub async fn serve_local_pipe(easytier: whalelink_core::EasyTierConfig) -> std::io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut daemon = ManagedDaemon::new(easytier);
    const PIPE: &str = r"\\.\pipe\WhaleLink.v1";
    loop {
        let mut pipe = create_current_user_pipe(PIPE)?;
        pipe.connect().await?;
        let mut buffer = [0_u8; 65_536];
        let read = pipe.read(&mut buffer).await?;
        let response = match serde_json::from_slice::<IpcRequest>(&buffer[..read]) {
            Ok(request) => daemon.dispatch(request).await,
            Err(_) => failure(
                Uuid::new_v4(),
                ErrorCode::ValidationFailed,
                "invalid IPC JSON",
            ),
        };
        pipe.write_all(
            serde_json::to_string(&response)
                .expect("IPC response is serializable")
                .as_bytes(),
        )
        .await?;
        pipe.write_all(b"\n").await?;
        pipe.flush().await?;
    }
}

/// Creates the pipe with a protected DACL that grants full access exclusively
/// to its owner (the user running the daemon). `reject_remote_clients` is kept
/// as a second, transport-level boundary.
#[cfg(windows)]
fn create_current_user_pipe(
    name: &str,
) -> std::io::Result<tokio::net::windows::named_pipe::NamedPipeServer> {
    use std::{ffi::c_void, mem::size_of, ptr};
    use tokio::net::windows::named_pipe::ServerOptions;
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            SECURITY_ATTRIBUTES,
        },
    };

    // OW is the SID of the token that creates this named pipe. The protected
    // DACL avoids inheriting broader permissions from a process environment.
    let descriptor: Vec<u16> = "D:P(A;;GA;;;OW)".encode_utf16().chain(Some(0)).collect();
    let mut security_descriptor: *mut c_void = ptr::null_mut();
    let converted = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            descriptor.as_ptr(),
            SDDL_REVISION_1,
            &mut security_descriptor,
            ptr::null_mut(),
        )
    };
    if converted == 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: security_descriptor,
        bInheritHandle: 0,
    };
    let result = unsafe {
        ServerOptions::new()
            .first_pipe_instance(true)
            .reject_remote_clients(true)
            .create_with_security_attributes_raw(
                name,
                (&mut attributes as *mut SECURITY_ATTRIBUTES).cast(),
            )
    };
    unsafe { LocalFree(security_descriptor) };
    result
}

#[cfg(windows)]
struct ManagedDaemon {
    easytier: whalelink_core::EasyTierConfig,
    process: Option<whalelink_core::EasyTierProcess>,
}

#[cfg(windows)]
#[derive(serde::Deserialize)]
struct ImportEnrollmentParams {
    room_id: String,
    enrollment: DataPlaneEnrollment,
}

#[cfg(windows)]
#[derive(serde::Deserialize)]
struct RoomParams {
    room_id: String,
}

#[cfg(windows)]
impl ManagedDaemon {
    fn new(easytier: whalelink_core::EasyTierConfig) -> Self {
        Self {
            easytier,
            process: None,
        }
    }

    async fn dispatch(&mut self, request: IpcRequest) -> IpcResponse {
        let request_id = request.request_id;
        if request.protocol_version != IPC_PROTOCOL_VERSION {
            return failure(
                request_id,
                ErrorCode::VersionIncompatible,
                "unsupported IPC protocol version",
            );
        }
        match request.method.as_str() {
            "version" => success(
                request_id,
                serde_json::json!({ "protocol_version": IPC_PROTOCOL_VERSION }),
            ),
            "health" | "status.get" => success(
                request_id,
                serde_json::json!({ "status": if self.process.is_some() { "running" } else { "stopped" } }),
            ),
            "enrollment.import" => {
                match serde_json::from_value::<ImportEnrollmentParams>(request.params) {
                    Ok(params) => match whalelink_core::EnrollmentStore::current_user()
                        .and_then(|store| store.save(&params.room_id, &params.enrollment))
                    {
                        Ok(()) => success(request_id, serde_json::json!({ "stored": true })),
                        Err(_) => failure(
                            request_id,
                            ErrorCode::Internal,
                            "could not store enrollment",
                        ),
                    },
                    Err(_) => failure(
                        request_id,
                        ErrorCode::ValidationFailed,
                        "invalid enrollment import request",
                    ),
                }
            }
            "tunnel.start" => match serde_json::from_value::<RoomParams>(request.params) {
                Ok(params) => self.start_tunnel(request_id, &params.room_id).await,
                Err(_) => failure(
                    request_id,
                    ErrorCode::ValidationFailed,
                    "room_id is required",
                ),
            },
            "tunnel.stop" => self.stop_tunnel(request_id).await,
            "diagnostics.export" => success(request_id, diagnostic_summary(self.process.is_some())),
            _ => failure(request_id, ErrorCode::NotFound, "unknown IPC method"),
        }
    }

    async fn start_tunnel(&mut self, request_id: Uuid, room_id: &str) -> IpcResponse {
        if self.process.is_some() {
            return failure(
                request_id,
                ErrorCode::Forbidden,
                "an EasyTier tunnel is already running",
            );
        }
        let enrollment = match whalelink_core::EnrollmentStore::current_user()
            .and_then(|store| store.load(room_id))
        {
            Ok(enrollment) => enrollment,
            Err(_) => {
                return failure(
                    request_id,
                    ErrorCode::NotFound,
                    "no protected enrollment for room",
                )
            }
        };
        let mut config = self.easytier.clone();
        config.arguments.extend(easytier_arguments(&enrollment));
        let mut process = whalelink_core::EasyTierProcess::new(config);
        match process.start().await {
            Ok(()) => {
                self.process = Some(process);
                success(request_id, serde_json::json!({ "status": "starting" }))
            }
            Err(_) => failure(request_id, ErrorCode::Internal, "could not start EasyTier"),
        }
    }

    async fn stop_tunnel(&mut self, request_id: Uuid) -> IpcResponse {
        let Some(mut process) = self.process.take() else {
            return failure(
                request_id,
                ErrorCode::NotFound,
                "no EasyTier tunnel is running",
            );
        };
        match process.stop().await {
            Ok(()) => success(request_id, serde_json::json!({ "status": "stopped" })),
            Err(_) => failure(request_id, ErrorCode::Internal, "could not stop EasyTier"),
        }
    }
}

fn diagnostic_summary(easytier_running: bool) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "redacted": true,
        "data_plane_status": if easytier_running { "running" } else { "stopped" },
        "contains_credentials": false,
        "contains_network_addresses": false,
    })
}

#[cfg(windows)]
fn easytier_arguments(enrollment: &DataPlaneEnrollment) -> Vec<String> {
    let mut arguments = vec![
        "--network-name".into(),
        enrollment.network_name.clone(),
        "--network-secret".into(),
        enrollment.network_secret.clone(),
        "--rpc-portal".into(),
        "127.0.0.1:0".into(),
        "--console-log-level".into(),
        "warn".into(),
    ];
    if enrollment.dhcp {
        arguments.push("--dhcp".into());
    }
    if enrollment.disable_upnp {
        arguments.push("--disable-upnp".into());
    }
    if !enrollment.peers.is_empty() {
        arguments.push("--peers".into());
        arguments.extend(enrollment.peers.iter().cloned());
    }
    arguments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incompatible_ipc_is_rejected() {
        let response = handle_ipc(
            IpcRequest {
                protocol_version: 99,
                request_id: Uuid::new_v4(),
                method: "health".into(),
                params: serde_json::Value::Null,
            },
            false,
        );
        assert_eq!(
            response.result.error.unwrap().code,
            ErrorCode::VersionIncompatible
        );
    }

    #[test]
    fn diagnostics_summary_contains_no_configuration_material() {
        let response = handle_ipc(
            IpcRequest {
                protocol_version: IPC_PROTOCOL_VERSION,
                request_id: Uuid::new_v4(),
                method: "diagnostics.export".into(),
                params: serde_json::Value::Null,
            },
            false,
        );
        let data = response.result.data.unwrap();
        assert_eq!(data["redacted"], true);
        assert_eq!(data["contains_credentials"], false);
        assert_eq!(data["contains_network_addresses"], false);
    }

    #[cfg(windows)]
    #[test]
    fn enrollment_arguments_enable_expected_safety_flags() {
        let arguments = easytier_arguments(&DataPlaneEnrollment {
            network_name: "test-network".into(),
            network_secret: "not-logged".into(),
            peers: vec!["tcp://relay.invalid:11010".into()],
            dhcp: true,
            disable_upnp: true,
        });
        assert!(arguments.contains(&"--dhcp".into()));
        assert!(arguments.contains(&"--disable-upnp".into()));
        assert!(arguments.contains(&"--peers".into()));
    }
}

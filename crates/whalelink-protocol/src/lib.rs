//! Stable data contracts shared by the daemon, CLI, server, and desktop client.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const API_VERSION: &str = "v1";
pub const IPC_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiEnvelope<T> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl<T> ApiEnvelope<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(error: ApiError) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub request_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Unauthorized,
    Forbidden,
    NotFound,
    ValidationFailed,
    InviteExpired,
    InviteConsumed,
    InviteRevoked,
    VersionIncompatible,
    ControlPlaneUnavailable,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoomSummary {
    pub id: String,
    pub display_name: String,
    pub member_count: u32,
    pub credential_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateInviteRequest {
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InviteCreated {
    pub id: Uuid,
    /// This code is returned exactly once to the administrator. It must not be logged.
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InviteRedemption {
    pub room_id: String,
    pub enrollment: EnrollmentMaterial,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnrollmentMaterial {
    pub credential_epoch: u64,
    /// Sensitive EasyTier configuration. Clients must protect it at rest and
    /// must never include it in logs or diagnostics.
    pub data_plane: DataPlaneEnrollment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPlaneEnrollment {
    pub network_name: String,
    pub network_secret: String,
    pub peers: Vec<String>,
    pub dhcp: bool,
    pub disable_upnp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemberSummary {
    pub id: String,
    pub joined_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub service: String,
    pub api_version: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcRequest {
    pub protocol_version: u32,
    pub request_id: Uuid,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpcResponse {
    pub protocol_version: u32,
    pub request_id: Uuid,
    #[serde(flatten)]
    pub result: ApiEnvelope<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_has_one_result_channel() {
        let encoded = serde_json::to_value(ApiEnvelope::success("ok")).unwrap();
        assert_eq!(encoded["ok"], true);
        assert_eq!(encoded["data"], "ok");
        assert!(encoded.get("error").is_none());
    }
}

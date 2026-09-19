use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path as FsPath, PathBuf},
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use whalelink_protocol::{
    ApiEnvelope, ApiError, CreateInviteRequest, DataPlaneEnrollment, EnrollmentMaterial, ErrorCode,
    HealthResponse, InviteCreated, InviteRedemption, MemberSummary, RoomSummary, API_VERSION,
};

#[derive(Clone)]
pub struct ServerState {
    admin_token: String,
    store: Arc<Mutex<Store>>,
    snapshot_path: Option<PathBuf>,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub rooms: Vec<ConfiguredRoom>,
}

#[derive(Deserialize)]
pub struct ConfiguredRoom {
    pub id: String,
    pub display_name: String,
    pub data_plane: ConfiguredDataPlane,
}

#[derive(Deserialize)]
pub struct ConfiguredDataPlane {
    pub network_name: String,
    pub network_secret_env: String,
    pub peers: Vec<String>,
    #[serde(default = "default_true")]
    pub dhcp: bool,
    #[serde(default = "default_true")]
    pub disable_upnp: bool,
}

#[derive(Clone)]
pub struct RoomDefinition {
    pub id: String,
    pub display_name: String,
    pub data_plane: DataPlaneEnrollment,
}

fn default_true() -> bool {
    true
}

pub fn load_rooms(path: &FsPath) -> Result<Vec<RoomDefinition>, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    load_rooms_from_toml(&contents, |variable| std::env::var(variable).ok())
}

fn load_rooms_from_toml(
    contents: &str,
    secret_resolver: impl Fn(&str) -> Option<String>,
) -> Result<Vec<RoomDefinition>, std::io::Error> {
    let config: ServerConfig = toml::from_str(contents)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    if config.rooms.is_empty()
        || config.rooms.iter().any(|room| {
            room.id.trim().is_empty()
                || room.display_name.trim().is_empty()
                || room.data_plane.network_name.trim().is_empty()
                || room.data_plane.network_secret_env.trim().is_empty()
                || room.data_plane.peers.is_empty()
                || room
                    .data_plane
                    .peers
                    .iter()
                    .any(|peer| peer.trim().is_empty())
        })
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "at least one room with id and display_name is required",
        ));
    }
    config
        .rooms
        .into_iter()
        .map(|room| {
            let network_secret =
                secret_resolver(&room.data_plane.network_secret_env).ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "environment variable {} is required",
                            room.data_plane.network_secret_env
                        ),
                    )
                })?;
            if network_secret.trim().is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "network secret must not be empty",
                ));
            }
            Ok(RoomDefinition {
                id: room.id,
                display_name: room.display_name,
                data_plane: DataPlaneEnrollment {
                    network_name: room.data_plane.network_name,
                    network_secret,
                    peers: room.data_plane.peers,
                    dhcp: room.data_plane.dhcp,
                    disable_upnp: room.data_plane.disable_upnp,
                },
            })
        })
        .collect::<Result<Vec<_>, std::io::Error>>()
}

#[derive(Default, Serialize, Deserialize)]
struct Store {
    schema_version: u32,
    rooms: HashMap<String, Room>,
    invites: HashMap<String, Invite>,
}

#[derive(Serialize, Deserialize)]
struct Room {
    id: String,
    display_name: String,
    credential_epoch: u64,
    members: Vec<MemberSummary>,
    #[serde(skip)]
    data_plane: Option<DataPlaneEnrollment>,
}

#[derive(Serialize, Deserialize)]
struct Invite {
    id: Uuid,
    room_id: String,
    expires_at: DateTime<Utc>,
    revoked: bool,
    consumed: bool,
}

impl ServerState {
    pub fn with_rooms(
        admin_token: impl Into<String>,
        rooms: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        let definitions = rooms.into_iter().map(|(id, display_name)| RoomDefinition {
            id,
            display_name,
            data_plane: DataPlaneEnrollment {
                network_name: String::new(),
                network_secret: String::new(),
                peers: Vec::new(),
                dhcp: true,
                disable_upnp: true,
            },
        });
        Self::with_room_definitions_without_enrollment(admin_token, definitions)
    }

    fn with_room_definitions_without_enrollment(
        admin_token: impl Into<String>,
        rooms: impl IntoIterator<Item = RoomDefinition>,
    ) -> Self {
        let mut store = Store {
            schema_version: 1,
            ..Store::default()
        };
        for definition in rooms {
            let id = definition.id;
            store.rooms.insert(
                id.clone(),
                Room {
                    id,
                    display_name: definition.display_name,
                    credential_epoch: 1,
                    members: Vec::new(),
                    data_plane: None,
                },
            );
        }
        Self {
            admin_token: admin_token.into(),
            store: Arc::new(Mutex::new(store)),
            snapshot_path: None,
        }
    }

    pub fn with_room_definitions(
        admin_token: impl Into<String>,
        rooms: impl IntoIterator<Item = RoomDefinition>,
    ) -> Self {
        let definitions: Vec<_> = rooms.into_iter().collect();
        let state =
            Self::with_room_definitions_without_enrollment(admin_token, definitions.clone());
        let mut store = state.store.lock().expect("store lock poisoned");
        for definition in definitions {
            store
                .rooms
                .get_mut(&definition.id)
                .expect("definition created the room")
                .data_plane = Some(definition.data_plane);
        }
        drop(store);
        state
    }

    pub fn from_or_create_state_file(
        admin_token: impl Into<String>,
        path: impl Into<PathBuf>,
        rooms: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Self, std::io::Error> {
        let path = path.into();
        let token = admin_token.into();
        let mut state = if path.exists() {
            Self::load_snapshot(token, &path)?
        } else {
            Self::with_rooms(token, rooms)
        };
        state.snapshot_path = Some(path);
        state.persist()?;
        Ok(state)
    }

    pub fn from_or_create_state_file_with_room_definitions(
        admin_token: impl Into<String>,
        path: impl Into<PathBuf>,
        rooms: impl IntoIterator<Item = RoomDefinition>,
    ) -> Result<Self, std::io::Error> {
        let path = path.into();
        let token = admin_token.into();
        let definitions: Vec<_> = rooms.into_iter().collect();
        let mut state = if path.exists() {
            Self::load_snapshot_with_room_definitions(token, &path, definitions)?
        } else {
            Self::with_room_definitions(token, definitions)
        };
        state.snapshot_path = Some(path);
        state.persist()?;
        Ok(state)
    }

    pub fn save_snapshot(&self, path: &FsPath) -> Result<(), std::io::Error> {
        let store = self.store.lock().expect("store lock poisoned");
        let parent = path.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "snapshot requires a parent directory",
            )
        })?;
        fs::create_dir_all(parent)?;
        let temporary = path.with_extension("json.tmp");
        fs::write(
            &temporary,
            serde_json::to_vec_pretty(&*store).expect("store is serializable"),
        )?;
        fs::rename(temporary, path)
    }

    pub fn load_snapshot(
        admin_token: impl Into<String>,
        path: &FsPath,
    ) -> Result<Self, std::io::Error> {
        let store: Store = serde_json::from_slice(&fs::read(path)?)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        if store.schema_version != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported server state schema",
            ));
        }
        Ok(Self {
            admin_token: admin_token.into(),
            store: Arc::new(Mutex::new(store)),
            snapshot_path: None,
        })
    }

    pub fn load_snapshot_with_room_definitions(
        admin_token: impl Into<String>,
        path: &FsPath,
        rooms: impl IntoIterator<Item = RoomDefinition>,
    ) -> Result<Self, std::io::Error> {
        let state = Self::load_snapshot(admin_token, path)?;
        let definitions: HashMap<_, _> = rooms
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect();
        let mut store = state.store.lock().expect("store lock poisoned");
        if store.rooms.len() != definitions.len()
            || store.rooms.keys().any(|id| !definitions.contains_key(id))
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "configured room IDs do not match persisted state",
            ));
        }
        for room in store.rooms.values_mut() {
            let definition = definitions.get(&room.id).expect("room key was validated");
            if room.display_name != definition.display_name {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "configured room display names do not match persisted state",
                ));
            }
            room.data_plane = Some(definition.data_plane.clone());
        }
        drop(store);
        Ok(state)
    }

    fn persist(&self) -> Result<(), std::io::Error> {
        match &self.snapshot_path {
            Some(path) => self.save_snapshot(path),
            None => Ok(()),
        }
    }

    fn create_invite(
        &self,
        room_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<InviteCreated, DomainError> {
        if expires_at <= Utc::now() {
            return Err(DomainError::Validation("expires_at must be in the future"));
        }
        let mut store = self.store.lock().expect("store lock poisoned");
        if !store.rooms.contains_key(room_id) {
            return Err(DomainError::NotFound);
        }
        let code = format!(
            "wl_{}",
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(40)
                .map(char::from)
                .collect::<String>()
        );
        let id = Uuid::new_v4();
        store.invites.insert(
            invite_hash(&code),
            Invite {
                id,
                room_id: room_id.to_owned(),
                expires_at,
                revoked: false,
                consumed: false,
            },
        );
        let created = InviteCreated {
            id,
            code,
            expires_at,
        };
        drop(store);
        self.persist().map_err(|_| DomainError::Persistence)?;
        Ok(created)
    }

    fn redeem(&self, code: &str) -> Result<InviteRedemption, DomainError> {
        let mut store = self.store.lock().expect("store lock poisoned");
        let room_id = {
            let invite = store
                .invites
                .get(&invite_hash(code))
                .ok_or(DomainError::NotFound)?;
            if invite.revoked {
                return Err(DomainError::Revoked);
            }
            if invite.consumed {
                return Err(DomainError::Consumed);
            }
            if invite.expires_at <= Utc::now() {
                return Err(DomainError::Expired);
            }
            invite.room_id.clone()
        };
        let data_plane = store
            .rooms
            .get(&room_id)
            .expect("invite always references a room")
            .data_plane
            .clone()
            .ok_or(DomainError::DataPlaneUnavailable)?;
        store
            .invites
            .get_mut(&invite_hash(code))
            .expect("invite was validated above")
            .consumed = true;
        let room = store
            .rooms
            .get_mut(&room_id)
            .expect("invite always references a room");
        let now = Utc::now();
        room.members.push(MemberSummary {
            id: Uuid::new_v4().to_string(),
            joined_at: now,
            last_seen_at: now,
        });
        let redemption = InviteRedemption {
            room_id: room_id.clone(),
            enrollment: EnrollmentMaterial {
                credential_epoch: room.credential_epoch,
                data_plane,
            },
        };
        drop(store);
        self.persist().map_err(|_| DomainError::Persistence)?;
        Ok(redemption)
    }

    fn revoke(&self, invite_id: Uuid) -> Result<(), DomainError> {
        let mut store = self.store.lock().expect("store lock poisoned");
        let invite = store
            .invites
            .values_mut()
            .find(|item| item.id == invite_id)
            .ok_or(DomainError::NotFound)?;
        invite.revoked = true;
        drop(store);
        self.persist().map_err(|_| DomainError::Persistence)?;
        Ok(())
    }

    fn rooms(&self) -> Vec<RoomSummary> {
        let store = self.store.lock().expect("store lock poisoned");
        store
            .rooms
            .values()
            .map(|room| RoomSummary {
                id: room.id.clone(),
                display_name: room.display_name.clone(),
                member_count: room.members.len() as u32,
                credential_epoch: room.credential_epoch,
            })
            .collect()
    }

    fn members(&self, room_id: &str) -> Result<Vec<MemberSummary>, DomainError> {
        let store = self.store.lock().expect("store lock poisoned");
        store
            .rooms
            .get(room_id)
            .map(|room| room.members.clone())
            .ok_or(DomainError::NotFound)
    }

    fn rotate(&self, room_id: &str) -> Result<RoomSummary, DomainError> {
        let mut store = self.store.lock().expect("store lock poisoned");
        let room = store.rooms.get_mut(room_id).ok_or(DomainError::NotFound)?;
        room.credential_epoch += 1;
        let summary = RoomSummary {
            id: room.id.clone(),
            display_name: room.display_name.clone(),
            member_count: room.members.len() as u32,
            credential_epoch: room.credential_epoch,
        };
        drop(store);
        self.persist().map_err(|_| DomainError::Persistence)?;
        Ok(summary)
    }
}

#[derive(Debug)]
enum DomainError {
    Validation(&'static str),
    NotFound,
    Expired,
    Consumed,
    Revoked,
    DataPlaneUnavailable,
    Persistence,
}

/// Invites are keyed by a one-way digest so a future persisted store never
/// contains a redeemable invite code in plaintext.
fn invite_hash(code: &str) -> String {
    format!("{:x}", Sha256::digest(code.as_bytes()))
}

pub fn router(state: ServerState) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/rooms", get(list_rooms))
        .route("/api/v1/rooms/:room_id/invites", post(create_invite))
        .route("/api/v1/invites/:code/redeem", post(redeem))
        .route("/api/v1/invites/:invite_id", delete(revoke))
        .route("/api/v1/rooms/:room_id/members", get(members))
        .route("/api/v1/rooms/:room_id/credentials/rotate", post(rotate))
        .with_state(state)
}

async fn health() -> Json<ApiEnvelope<HealthResponse>> {
    Json(ApiEnvelope::success(HealthResponse {
        service: "whalelink-server".into(),
        api_version: API_VERSION.into(),
        status: "ok".into(),
    }))
}

async fn list_rooms(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<RoomSummary>>>, AppError> {
    authorize(&headers, &state)?;
    Ok(Json(ApiEnvelope::success(state.rooms())))
}

async fn create_invite(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Path(room_id): Path<String>,
    Json(request): Json<CreateInviteRequest>,
) -> Result<Json<ApiEnvelope<InviteCreated>>, AppError> {
    authorize(&headers, &state)?;
    Ok(Json(ApiEnvelope::success(
        state.create_invite(&room_id, request.expires_at)?,
    )))
}

async fn redeem(
    State(state): State<ServerState>,
    Path(code): Path<String>,
) -> Result<Json<ApiEnvelope<InviteRedemption>>, AppError> {
    Ok(Json(ApiEnvelope::success(state.redeem(&code)?)))
}

async fn revoke(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Path(invite_id): Path<Uuid>,
) -> Result<Json<ApiEnvelope<()>>, AppError> {
    authorize(&headers, &state)?;
    state.revoke(invite_id)?;
    Ok(Json(ApiEnvelope::success(())))
}

async fn members(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Path(room_id): Path<String>,
) -> Result<Json<ApiEnvelope<Vec<MemberSummary>>>, AppError> {
    authorize(&headers, &state)?;
    Ok(Json(ApiEnvelope::success(state.members(&room_id)?)))
}

async fn rotate(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Path(room_id): Path<String>,
) -> Result<Json<ApiEnvelope<RoomSummary>>, AppError> {
    authorize(&headers, &state)?;
    Ok(Json(ApiEnvelope::success(state.rotate(&room_id)?)))
}

fn authorize(headers: &HeaderMap, state: &ServerState) -> Result<(), AppError> {
    let supplied = headers
        .get("X-WhaleLink-Token")
        .and_then(|value| value.to_str().ok());
    if supplied == Some(state.admin_token.as_str()) {
        Ok(())
    } else {
        Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            ErrorCode::Unauthorized,
            "administrator token required",
        ))
    }
}

struct AppError {
    status: StatusCode,
    code: ErrorCode,
    message: String,
    request_id: Uuid,
}
impl AppError {
    fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            request_id: Uuid::new_v4(),
        }
    }
}
impl From<DomainError> for AppError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::Validation(message) => Self::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationFailed,
                message,
            ),
            DomainError::NotFound => Self::new(
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "resource not found",
            ),
            DomainError::Expired => Self::new(
                StatusCode::GONE,
                ErrorCode::InviteExpired,
                "invite has expired",
            ),
            DomainError::Consumed => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::InviteConsumed,
                "invite has already been redeemed",
            ),
            DomainError::Revoked => Self::new(
                StatusCode::GONE,
                ErrorCode::InviteRevoked,
                "invite has been revoked",
            ),
            DomainError::DataPlaneUnavailable => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::ControlPlaneUnavailable,
                "room enrollment material is unavailable",
            ),
            DomainError::Persistence => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "state persistence failed",
            ),
        }
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiEnvelope::<()>::failure(ApiError {
                code: self.code,
                message: self.message,
                request_id: self.request_id,
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use chrono::Duration;
    use tower::ServiceExt;

    fn room_definition() -> RoomDefinition {
        RoomDefinition {
            id: "alpha".into(),
            display_name: "Alpha room".into(),
            data_plane: DataPlaneEnrollment {
                network_name: "wl-test-alpha".into(),
                network_secret: "test-secret-must-never-persist-0123456789".into(),
                peers: vec!["tcp://relay.invalid:11010".into()],
                dhcp: true,
                disable_upnp: true,
            },
        }
    }

    fn state() -> ServerState {
        ServerState::with_room_definitions("test-admin-token", [room_definition()])
    }

    #[test]
    fn invite_is_single_use() {
        let state = state();
        let invite = state
            .create_invite("alpha", Utc::now() + Duration::minutes(5))
            .unwrap();
        assert!(state.redeem(&invite.code).is_ok());
        assert!(matches!(
            state.redeem(&invite.code),
            Err(DomainError::Consumed)
        ));
    }

    #[test]
    fn store_does_not_index_invites_by_plaintext_code() {
        let state = state();
        let invite = state
            .create_invite("alpha", Utc::now() + Duration::minutes(5))
            .unwrap();
        let store = state.store.lock().unwrap();
        assert!(!store.invites.contains_key(&invite.code));
        assert!(store.invites.contains_key(&invite_hash(&invite.code)));
    }

    #[test]
    fn snapshot_restores_room_state_without_plaintext_invite_code() {
        let state = state();
        let invite = state
            .create_invite("alpha", Utc::now() + Duration::minutes(5))
            .unwrap();
        let path = std::env::temp_dir().join(format!("whalelink-state-{}.json", Uuid::new_v4()));
        state.save_snapshot(&path).unwrap();
        let snapshot = std::fs::read_to_string(&path).unwrap();
        assert!(!snapshot.contains(&invite.code));
        assert!(!snapshot.contains("test-secret-must-never-persist"));
        let restored = ServerState::load_snapshot("test-admin-token", &path).unwrap();
        assert_eq!(restored.rooms()[0].id, "alpha");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn configured_state_file_persists_new_invites() {
        let path = std::env::temp_dir().join(format!("whalelink-auto-{}.json", Uuid::new_v4()));
        let state = ServerState::from_or_create_state_file_with_room_definitions(
            "test-admin-token",
            &path,
            [room_definition()],
        )
        .unwrap();
        let invite = state
            .create_invite("alpha", Utc::now() + Duration::minutes(5))
            .unwrap();
        let restored = ServerState::from_or_create_state_file_with_room_definitions(
            "test-admin-token",
            &path,
            [room_definition()],
        )
        .unwrap();
        assert!(restored.redeem(&invite.code).is_ok());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn loads_fixed_rooms_from_toml() {
        let rooms = load_rooms_from_toml(
            "[[rooms]]\nid = 'alpha'\ndisplay_name = 'Alpha'\n[rooms.data_plane]\nnetwork_name = 'wl-alpha'\nnetwork_secret_env = 'TEST_ROOM_SECRET'\npeers = ['tcp://relay.invalid:11010']\n",
            |key| (key == "TEST_ROOM_SECRET").then(|| "0123456789abcdef0123456789abcdef".into()),
        )
        .unwrap();
        assert_eq!(rooms[0].id, "alpha");
        assert_eq!(rooms[0].data_plane.network_name, "wl-alpha");
    }

    #[test]
    fn persisted_state_rehydrates_enrollment_without_serializing_secret() {
        let path =
            std::env::temp_dir().join(format!("whalelink-rehydrate-{}.json", Uuid::new_v4()));
        let state = ServerState::from_or_create_state_file_with_room_definitions(
            "test-admin-token",
            &path,
            [room_definition()],
        )
        .unwrap();
        let invite = state
            .create_invite("alpha", Utc::now() + Duration::minutes(5))
            .unwrap();
        let snapshot = std::fs::read_to_string(&path).unwrap();
        assert!(!snapshot.contains("test-secret-must-never-persist"));
        let restored = ServerState::from_or_create_state_file_with_room_definitions(
            "test-admin-token",
            &path,
            [room_definition()],
        )
        .unwrap();
        assert_eq!(
            restored
                .redeem(&invite.code)
                .unwrap()
                .enrollment
                .data_plane
                .network_name,
            "wl-test-alpha"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn revoked_invite_cannot_be_redeemed() {
        let state = state();
        let invite = state
            .create_invite("alpha", Utc::now() + Duration::minutes(5))
            .unwrap();
        state.revoke(invite.id).unwrap();
        assert!(matches!(
            state.redeem(&invite.code),
            Err(DomainError::Revoked)
        ));
    }

    #[test]
    fn rotation_increments_epoch() {
        let state = state();
        assert_eq!(state.rotate("alpha").unwrap().credential_epoch, 2);
    }

    #[tokio::test]
    async fn rooms_route_requires_administrator_token() {
        let app = router(state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/rooms")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

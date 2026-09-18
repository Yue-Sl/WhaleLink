use std::{env, net::SocketAddr, path::PathBuf};
use whalelink_server::{load_rooms, router, ServerState};

#[tokio::main]
async fn main() {
    let admin_token = env::var("WHALELINK_ADMIN_TOKEN")
        .expect("WHALELINK_ADMIN_TOKEN must be set; do not use a development default");
    let bind = env::var("WHALELINK_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_owned());
    let address: SocketAddr = bind
        .parse()
        .expect("WHALELINK_BIND must be a socket address");
    let config_path = env::var("WHALELINK_CONFIG")
        .expect("WHALELINK_CONFIG must point to a fixed-room configuration");
    let rooms = load_rooms(&PathBuf::from(config_path)).expect("failed to load WHALELINK_CONFIG");
    let state = match env::var("WHALELINK_STATE_PATH") {
        Ok(path) => ServerState::from_or_create_state_file_with_room_definitions(
            admin_token,
            PathBuf::from(path),
            rooms,
        )
        .expect("failed to load or create WHALELINK_STATE_PATH"),
        Err(_) => ServerState::with_room_definitions(admin_token, rooms),
    };
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("failed to bind control plane listener");
    axum::serve(listener, router(state))
        .await
        .expect("control plane terminated unexpectedly");
}

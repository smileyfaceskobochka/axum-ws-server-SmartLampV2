mod models;
mod handlers;
mod utils;

use axum::{Router, routing::get, response::Redirect};
use tower_http::services::ServeDir;
use std::sync::Arc;
use handlers::*;
use models::AppState;

const SERVER_ADDRESS: &str = "0.0.0.0:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("axum_ws_server=debug,tower_http=info")
        .init();

    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/static/") }))
        .route("/ws/device", get(handle_device_ws_upgrade))
        .route("/ws/client", get(handle_client_ws_upgrade))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDRESS).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
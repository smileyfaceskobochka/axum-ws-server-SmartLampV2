// main.rs

mod handlers;
mod models;
mod utils;

use axum::{Router, response::Redirect, routing::get};
use handlers::*;
use models::AppState;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tower_http::services::ServeDir;

const SERVER_ADDRESS: &str = "0.0.0.0:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("axum_ws_server=debug,tower_http=info")
        .init();

    let state = Arc::new(AppState::new());

    // Start device status checker
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            utils::check_device_statuses(state_clone.clone()).await;
        }
    });

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

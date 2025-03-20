// main.rs
mod commands;
mod config;
mod devices;
mod events;
mod docs;
mod error;
mod handlers;
mod models;
mod utils;

use axum::{Router, routing::get, response::Redirect};
use tower_http::services::ServeDir;
use std::sync::Arc;
use handlers::*;
use models::AppState;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = config::Settings::new()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/static/") }))
        .route("/ws/device", get(handle_device_ws_upgrade))
        .route("/ws/client", get(handle_client_ws_upgrade))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", docs::ApiDoc::openapi()))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(&settings.server.address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind address: {}", e))?;
    
    tracing::info!("Server started on {}", settings.server.address);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
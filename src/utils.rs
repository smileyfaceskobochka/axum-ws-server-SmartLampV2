use super::models::AppState;
use uuid::Uuid;
use tracing::info;

pub async fn cleanup_device_connection(device_id: &str, state: &AppState) {
    state.devices.lock().await.remove(device_id);
    info!("Device {} disconnected", device_id);
}

pub async fn cleanup_client_connection(client_id: Uuid, state: &AppState) {
    state.clients.lock().await.remove(&client_id);
    info!("Client {} disconnected", client_id);
}
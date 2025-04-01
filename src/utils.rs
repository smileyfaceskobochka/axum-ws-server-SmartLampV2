// utils.rs

use super::models::{AppState, WsMessage};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub async fn cleanup_device_connection(device_id: &str, state: &AppState) {
    state.devices.remove(device_id);
    info!("Device {} disconnected", device_id);
}

pub async fn cleanup_client_connection(client_id: Uuid, state: &AppState) {
    state.clients.remove(&client_id);
    info!("Client {} disconnected", client_id);
}

pub async fn check_device_statuses(state: Arc<AppState>) {
    let mut devices_to_remove = Vec::new();

    for device in state.devices.iter() {
        let device_id = device.key().clone();
        let entry = device.value();

        // Send status request
        let status_request = WsMessage::StatusRequest {
            device_id: device_id.clone(),
        };
        if let Err(_) = entry.tx.send(status_request) {
            devices_to_remove.push(device_id);
        }
    }

    for device_id in devices_to_remove {
        state.devices.remove(&device_id);
        tracing::warn!("Device {} unresponsive, removed", device_id);
    }
}

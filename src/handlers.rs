// handlers.rs
use crate::{
    models::{AppState, DeviceEntry, DeviceStatus, WsMessage},
    utils,
};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::Instant;
use tracing::{error, info};
use uuid::Uuid;

pub async fn handle_device_ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Device connection attempt");
    ws.on_upgrade(|socket| handle_device(socket, state))
}

pub async fn handle_client_ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Client connection attempt");
    ws.on_upgrade(|socket| handle_client(socket, state))
}

async fn handle_device(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender)); // Wrap sender in Arc and Mutex
    let mut device_id = None;

    // Check if device is already registered
    if let Some(existing_entry) = state.devices.get(device_id.as_ref().unwrap_or(&String::new())) {
        // Send status request to existing device
        let status_request = WsMessage::StatusRequest { device_id: existing_entry.status.lock().await.device_id.clone() };
        if let Err(_) = existing_entry.tx.send(status_request) {
            // Remove unresponsive device
            state.devices.remove(existing_entry.status.lock().await.device_id.as_str());
        }
    }

    while let Some(Ok(msg)) = receiver.next().await {
        match msg.to_text() {
            Ok(text) => match serde_json::from_str::<WsMessage>(text) {
                Ok(WsMessage::DeviceRegistration { device_id: id }) => {
                    info!(%id, "Device registered");
                    let response = WsMessage::DeviceRegistered {
                        device_id: id.clone(),
                    };
                    let mut sender_lock = sender.lock().await;
                    if sender_lock
                        .send(Message::Text(
                            serde_json::to_string(&response).unwrap().into(),
                        ))
                        .await
                        .is_err()
                    {
                        return;
                    }
                    let (tx, _) = broadcast::channel(100);
                    let initial_status = DeviceStatus {
                        device_id: id.clone(),
                        power: false,
                        brightness: 0,
                        color: [0, 0, 0],
                        auto_brightness: false,
                        pos: [0, 0, 0, 0],
                        auto_pos: false,
                    };
                    let entry = DeviceEntry {
                        tx: tx.clone(),
                        status: Arc::new(Mutex::new(initial_status)),
                        last_seen: Arc::new(Mutex::new(Instant::now())),
                    };
                    state.devices.insert(id.clone(), entry);
                    device_id = Some(id);
                    break;
                }
                _ => {
                    let error = WsMessage::Error {
                        message: "Invalid registration".into(),
                        code: 400,
                    };
                    let mut sender_lock = sender.lock().await;
                    let _ = sender_lock
                        .send(Message::Text(serde_json::to_string(&error).unwrap().into()))
                        .await;
                }
            },
            Err(e) => {
                error!("Message parse error: {}", e);
                return;
            }
        }
    }

    if let Some(id) = device_id {
        let entry = state.devices.get(&id).unwrap();
        let mut rx = entry.tx.subscribe();

        let sender_clone = Arc::clone(&sender);
        let status_task = tokio::spawn(async move {
            let mut sender_lock = sender_clone.lock().await;
            while let Ok(msg) = rx.recv().await {
                if sender_lock
                    .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        let state_clone = Arc::clone(&state);
        let command_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Ok(text) = msg.to_text() {
                    if let Ok(WsMessage::StatusUpdate(status)) = serde_json::from_str(text) {
                        let clients = state_clone.clients.iter().collect::<Vec<_>>();
                        for client in clients {
                            let _ = client.value().send(WsMessage::StatusUpdate(status.clone()));
                        }
                    }
                }
            }
        });

        tokio::pin!(status_task, command_task);
        tokio::select! {
            _ = &mut status_task => command_task.abort(),
            _ = &mut command_task => status_task.abort(),
        };

        if let Some(id) = device_id {
            utils::cleanup_device_connection(&id, &state).await;
        }
    }
}

async fn handle_client(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender)); // Wrap sender in Arc and Mutex
    let client_id = Uuid::new_v4();
    let (tx, rx) = broadcast::channel(100);
    state.clients.insert(client_id, tx.clone());

    let send_task = tokio::spawn(async move {
        let mut rx = rx;
        while let Ok(msg) = rx.recv().await {
            let mut sender_lock = sender.lock().await;
            if sender_lock
                .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let recv_task = tokio::spawn({
        let state = Arc::clone(&state);
        async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Ok(text) = msg.to_text() {
                    if let Ok(command) = serde_json::from_str::<WsMessage>(text) {
                        match command {
                            WsMessage::SetPower { ref device_id, .. }
                            | WsMessage::SetBrightness { ref device_id, .. }
                            | WsMessage::SetColor { ref device_id, .. }
                            | WsMessage::SetAutoBrightness { ref device_id, .. }
                            | WsMessage::SetPosition { ref device_id, .. }
                            | WsMessage::SetAutoPosition { ref device_id, .. } => {
                                if let Some(entry) = state.devices.get(device_id) {
                                    let _ = entry.tx.send(command);
                                } else {
                                    let error = WsMessage::Error {
                                        message: "Device not found".into(),
                                        code: 404,
                                    };
                                    let _ = tx.send(error);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    });

    tokio::pin!(send_task, recv_task);
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    utils::cleanup_client_connection(client_id, &state).await;
}
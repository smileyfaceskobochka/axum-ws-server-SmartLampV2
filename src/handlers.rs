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
use tokio::sync::{Mutex, broadcast};
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
    let sender = Arc::new(Mutex::new(sender));
    let mut device_id: Option<String> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        // Обновляем время активности
        if let Some(ref id) = device_id {
            if let Some(entry) = state.devices.get(id.as_str()) {
                *entry.last_activity.lock().await = Instant::now();
            }
        }

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
                        last_activity: Arc::new(Mutex::new(Instant::now())),
                    };

                    state.devices.insert(id.clone(), entry);
                    device_id = Some(id);
                    break;
                }
                Ok(_message) => {
                    // Обработка других сообщений
                }
                Err(e) => {
                    error!("Invalid message format: {}", e);
                    return;
                }
            },
            Err(e) => {
                error!("Message parse error: {}", e);
                return;
            }
        }
    }

    if let Some(ref id) = device_id {
        let entry = state.devices.get(id.as_str()).unwrap();
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
        let recv_task = tokio::spawn(async move {
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

        tokio::pin!(status_task, recv_task);

        tokio::select! {
            _ = &mut status_task => recv_task.abort(),
            _ = &mut recv_task => status_task.abort(),
        };

        utils::cleanup_device_connection(id, &state).await;
    }
}

async fn handle_client(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
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

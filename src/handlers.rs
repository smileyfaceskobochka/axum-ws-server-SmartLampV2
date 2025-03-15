use axum::{
    extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;
use crate::{
    models::{AppState, WsMessage},
    utils::{cleanup_client_connection, cleanup_device_connection} // Исправлены имена функций
};

pub async fn handle_device_ws_upgrade(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Device connection attempt");
    ws.on_upgrade(|socket| handle_device(socket, state))
}

pub async fn handle_client_ws_upgrade(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Client connection attempt");
    ws.on_upgrade(|socket| handle_client(socket, state))
}

async fn handle_device(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut device_id = None;

    while let Some(Ok(msg)) = receiver.next().await {
        match msg.to_text() {
            Ok(text) => {
                match serde_json::from_str::<WsMessage>(text) {
                    Ok(WsMessage::DeviceRegistration { device_id: id }) => {
                        info!(%id, "Device registered");
                        let response = WsMessage::DeviceRegistered { device_id: id.clone() };
                        if sender.send(Message::Text(serde_json::to_string(&response).unwrap().into())).await.is_err() {
                            return;
                        }
                        let (tx, _) = broadcast::channel(100);
                        state.devices.lock().await.insert(id.clone(), tx);
                        device_id = Some(id);
                        break;
                    }
                    _ => {
                        let error = WsMessage::Error { message: "Invalid registration".into(), code: 400 };
                        let _ = sender.send(Message::Text(serde_json::to_string(&error).unwrap().into())).await;
                    }
                }
            }
            Err(e) => {
                error!("Message parse error: {}", e);
                return;
            }
        }
    }

    let Some(id) = device_id else { return };

    let mut rx = state.devices.lock().await.get(&id).unwrap().subscribe();
    let status_task = tokio::spawn({
        let mut sender = sender;
        async move {
            while let Ok(msg) = rx.recv().await {
                if sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let command_task = tokio::spawn({
        let state = Arc::clone(&state);
        async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Ok(text) = msg.to_text() {
                    if let Ok(WsMessage::StatusUpdate(status)) = serde_json::from_str(text) {
                        let clients = state.clients.lock().await;
                        for client in clients.values() {
                            let _ = client.send(WsMessage::StatusUpdate(status.clone()));
                        }
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

    cleanup_device_connection(&id, &state).await;
}

async fn handle_client(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4();
    let (tx, mut rx) = broadcast::channel(100);
    state.clients.lock().await.insert(client_id, tx.clone());

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await.is_err() {
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
                        let devices = state.devices.lock().await;
                        match command {
                            WsMessage::SetPower { ref device_id, .. } 
                            | WsMessage::SetBrightness { ref device_id, .. }
                            | WsMessage::SetColor { ref device_id, .. } => {
                                if let Some(tx) = devices.get(device_id) {
                                    let _ = tx.send(command);
                                } else {
                                    let error = WsMessage::Error { message: "Device not found".into(), code: 404 };
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

    cleanup_client_connection(client_id, &state).await;
}
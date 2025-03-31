// models.rs

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_id: String,
    pub power: bool,
    pub brightness: u8,
    pub color: [u8; 3],
    pub auto_brightness: bool,
    pub pos: [u8; 4],
    pub auto_pos: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    DeviceRegistration {
        device_id: String,
    },
    DeviceRegistered {
        device_id: String,
    },
    StatusUpdate(DeviceStatus),
    SetPower {
        device_id: String,
        power: bool,
    },
    SetBrightness {
        device_id: String,
        brightness: u8,
    },
    SetColor {
        device_id: String,
        color: [u8; 3],
    },
    SetAutoBrightness {
        device_id: String,
        auto_brightness: bool,
    },
    SetPosition {
        device_id: String,
        pos: [u8; 4],
    },
    SetAutoPosition {
        device_id: String,
        auto_pos: bool,
    },
    StatusRequest {
        device_id: String,
    },
    Error {
        message: String,
        code: u16,
    },
}

pub struct DeviceEntry {
    pub tx: broadcast::Sender<WsMessage>,
    pub status: Arc<Mutex<DeviceStatus>>,
    pub last_seen: Arc<Mutex<Instant>>,
}

pub struct AppState {
    pub devices: DashMap<String, DeviceEntry>,
    pub clients: DashMap<uuid::Uuid, broadcast::Sender<WsMessage>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            devices: DashMap::new(),
            clients: DashMap::new(),
        }
    }
}

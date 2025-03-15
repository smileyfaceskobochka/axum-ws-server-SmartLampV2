use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_id: String,
    pub power: bool,
    pub brightness: u8,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    DeviceRegistration { device_id: String },
    DeviceRegistered { device_id: String },
    StatusUpdate(DeviceStatus),
    SetPower { device_id: String, power: bool },
    SetBrightness { device_id: String, brightness: u8 },
    SetColor { device_id: String, color: [u8; 3] },
    Error { message: String, code: u16 },
}

#[derive(Debug)]
pub struct AppState {
    pub devices: Mutex<HashMap<String, broadcast::Sender<WsMessage>>>,
    pub clients: Mutex<HashMap<Uuid, broadcast::Sender<WsMessage>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            devices: Mutex::new(HashMap::new()),
            clients: Mutex::new(HashMap::new()),
        }
    }
}
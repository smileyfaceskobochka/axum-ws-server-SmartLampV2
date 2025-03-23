// models.rs
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeviceStatus {
    pub device_id: String,
    pub power: bool,
    pub brightness: u8,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
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

#[derive(Clone)]
pub struct DeviceEntry {
    pub tx: broadcast::Sender<WsMessage>,
    pub status: Arc<RwLock<DeviceStatus>>,
}

#[derive(Debug, Validate)]
struct ColorValidation {
    #[validate(range(min = 0, max = 255))]
    r: u8,
    #[validate(range(min = 0, max = 255))]
    g: u8,
    #[validate(range(min = 0, max = 255))]
    b: u8,
}

impl WsMessage {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            WsMessage::SetColor { color, .. } => {
                ColorValidation {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                }.validate()
            }
            _ => Ok(())
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub devices: DashMap<String, DeviceEntry>,
    pub clients: DashMap<Uuid, broadcast::Sender<WsMessage>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            devices: DashMap::new(),
            clients: DashMap::new(),
        }
    }
}
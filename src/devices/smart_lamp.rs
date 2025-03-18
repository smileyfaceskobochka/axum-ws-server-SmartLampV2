// smart_lamp.rs
use tokio::sync::RwLock;
use async_trait::async_trait;
use crate::{models::{WsMessage, DeviceStatus}, error::AppError};

pub struct SmartLamp {
    power: RwLock<bool>,
    brightness: RwLock<u8>,
    color: RwLock<[u8; 3]>,
}

impl SmartLamp {
    pub fn new() -> Self {
        Self {
            power: RwLock::new(false),
            brightness: RwLock::new(0),
            color: RwLock::new([255, 255, 255]),
        }
    }
}

#[async_trait]
impl super::Device for SmartLamp {
    async fn handle_command(&self, command: WsMessage) -> Result<(), AppError> {
        match command {
            WsMessage::SetPower { power, .. } => {
                *self.power.write().await = power;
                Ok(())
            }
            WsMessage::SetBrightness { brightness, .. } => {
                *self.brightness.write().await = brightness;
                Ok(())
            }
            WsMessage::SetColor { color, .. } => {
                *self.color.write().await = color;
                Ok(())
            }
            _ => Err(AppError::UnsupportedCommand),
        }
    }

    async fn get_status(&self) -> DeviceStatus {
        DeviceStatus {
            device_id: "smart_lamp_001".to_string(),
            power: *self.power.read().await,
            brightness: *self.brightness.read().await,
            color: *self.color.read().await,
        }
    }
}
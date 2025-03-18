// commands/mod.rs
use crate::{AppState, error::AppError};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, state: Arc<AppState>, device_id: &str) -> Result<(), AppError>;
}

pub trait CommandFactory: Sync {
    fn create(&self, payload: &serde_json::Value) -> anyhow::Result<Box<dyn CommandHandler>>;
}

pub struct SetPowerCommand {
    power: bool,
}

inventory::collect!(&'static dyn CommandFactory);

#[async_trait]
impl CommandHandler for SetPowerCommand {
    async fn execute(&self, state: Arc<AppState>, device_id: &str) -> Result<(), AppError> {
        let device = state
            .devices
            .get(device_id)
            .ok_or(AppError::DeviceNotFound)?;

        let mut status = device.status.write().await;
        status.power = self.power;

        Ok(())
    }
}

pub struct SetPowerCommandFactory;

impl CommandFactory for SetPowerCommandFactory {
    fn create(&self, payload: &serde_json::Value) -> anyhow::Result<Box<dyn CommandHandler>> {
        let power = payload
            .get("power")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Missing power field"))?;
        Ok(Box::new(SetPowerCommand { power }))
    }
}

fn create_set_power_command(payload: &serde_json::Value) -> anyhow::Result<SetPowerCommand> {
    let power = payload
        .get("power")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| anyhow::anyhow!("Missing power field"))?;
    Ok(SetPowerCommand { power })
}

inventory::submit! {
    &SetPowerCommandFactory as &'static dyn CommandFactory
}

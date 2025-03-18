// devices/mod.rs
mod smart_lamp;
// pub use smart_lamp::SmartLamp;

#[async_trait::async_trait]
pub trait Device: Send + Sync {
    async fn handle_command(&self, command: crate::models::WsMessage) -> Result<(), crate::error::AppError>;
    async fn get_status(&self) -> crate::models::DeviceStatus;
}
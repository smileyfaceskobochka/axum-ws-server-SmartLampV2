// error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Unsupported command")]
    UnsupportedCommand,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}
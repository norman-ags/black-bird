use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    #[error("Storage error: {message}")]
    Storage { message: String },
    
    #[error("API error: {message}")]
    Api { message: String, status_code: Option<u16> },
    
    #[error("Authentication error: {message}")]
    Authentication { message: String },
    
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Schedule error: {message}")]
    Schedule { message: String },
    
    #[error("Encryption error: {message}")]
    Encryption { message: String },
    
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("System error: {message}")]
    System { message: String },
    
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl AppError {
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage { message: message.into() }
    }
    
    pub fn api(message: impl Into<String>, status_code: Option<u16>) -> Self {
        Self::Api { message: message.into(), status_code }
    }
    
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication { message: message.into() }
    }
    
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation { field: field.into(), message: message.into() }
    }
    
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration { message: message.into() }
    }
    
    pub fn schedule(message: impl Into<String>) -> Self {
        Self::Schedule { message: message.into() }
    }
    
    pub fn encryption(message: impl Into<String>) -> Self {
        Self::Encryption { message: message.into() }
    }
    
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network { message: message.into() }
    }
    
    pub fn system(message: impl Into<String>) -> Self {
        Self::System { message: message.into() }
    }
    
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown { message: message.into() }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::system(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::system(format!("JSON error: {}", err))
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        AppError::system(format!("Tauri error: {}", err))
    }
}

pub fn setup_error_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let error = AppError::system(format!("Panic occurred: {}", panic_info));
        println!("CRITICAL ERROR: {}", error);
    }));
}

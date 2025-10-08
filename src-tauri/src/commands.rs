use crate::errors::AppError;
use crate::storage::create_storage_backend;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageResult {
    pub success: bool,
    pub message: String,
}

pub type StorageError = AppError;

fn validate_storage_key(key: &str) -> Result<(), StorageError> {
    if key.is_empty() {
        return Err(AppError::validation("key", "Storage key cannot be empty"));
    }
    if key.len() > 100 {
        return Err(AppError::validation("key", "Storage key too long"));
    }
    let invalid_chars = ['/', '\\', '<', '>', ':', '"', '|', '?', '*'];
    if key.chars().any(|c| invalid_chars.contains(&c) || c.is_control()) {
        return Err(AppError::validation("key", "Storage key contains invalid characters"));
    }
    Ok(())
}

#[tauri::command]
pub async fn store_encrypted_data(
    app_handle: AppHandle,
    key: String,
    encrypted_data: String,
) -> Result<StorageResult, String> {
    validate_storage_key(&key).map_err(|e| format!("Key validation failed: {}", e))?;
    if encrypted_data.is_empty() {
        return Err("Encrypted data cannot be empty".to_string());
    }
    let storage = create_storage_backend(app_handle).map_err(|e| format!("Failed to create storage backend: {}", e))?;
    storage.store(&key, &encrypted_data).await.map_err(|e| format!("Storage operation failed: {}", e))
}

#[tauri::command]
pub async fn retrieve_encrypted_data(
    app_handle: AppHandle,
    key: String,
) -> Result<Option<String>, String> {
    validate_storage_key(&key).map_err(|e| format!("Key validation failed: {}", e))?;
    let storage = create_storage_backend(app_handle).map_err(|e| format!("Failed to create storage backend: {}", e))?;
    storage.retrieve(&key).await.map_err(|e| format!("Retrieval operation failed: {}", e))
}

#[tauri::command]
pub async fn delete_encrypted_data(
    app_handle: AppHandle,
    key: String,
) -> Result<StorageResult, String> {
    validate_storage_key(&key).map_err(|e| format!("Key validation failed: {}", e))?;
    let storage = create_storage_backend(app_handle).map_err(|e| format!("Failed to create storage backend: {}", e))?;
    storage.delete(&key).await.map_err(|e| format!("Delete operation failed: {}", e))
}

#[tauri::command]
pub async fn list_storage_keys(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let storage = create_storage_backend(app_handle).map_err(|e| format!("Failed to create storage backend: {}", e))?;
    storage.list_keys().await.map_err(|e| format!("List operation failed: {}", e))
}

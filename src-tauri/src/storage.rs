use crate::commands::{StorageError, StorageResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub app_data_dir: PathBuf,
    pub encryption_enabled: bool,
}

pub struct StorageBackend {
    config: StorageConfig,
    app_handle: AppHandle,
}

impl StorageBackend {
    pub fn new(app_handle: AppHandle) -> Result<Self, StorageError> {
        let app_data_dir = app_handle.path().app_data_dir().map_err(StorageError::from)?;
        fs::create_dir_all(&app_data_dir)?;
        
        let config = StorageConfig {
            app_data_dir,
            encryption_enabled: true,
        };
        
        Ok(Self { config, app_handle })
    }
    
    fn get_file_path(&self, key: &str) -> PathBuf {
        let extension = if self.config.encryption_enabled { "enc" } else { "json" };
        self.config.app_data_dir.join(format!("{}.{}", key, extension))
    }
    
    pub async fn store(&self, key: &str, data: &str) -> Result<StorageResult, StorageError> {
        let file_path = self.get_file_path(key);
        fs::write(&file_path, data)?;
        Ok(StorageResult {
            success: true,
            message: format!("Data stored successfully: {:?}", file_path),
        })
    }
    
    pub async fn retrieve(&self, key: &str) -> Result<Option<String>, StorageError> {
        let file_path = self.get_file_path(key);
        if !file_path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&file_path)?;
        Ok(Some(data))
    }
    
    pub async fn delete(&self, key: &str) -> Result<StorageResult, StorageError> {
        let file_path = self.get_file_path(key);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
            Ok(StorageResult {
                success: true,
                message: format!("Data deleted: {:?}", file_path),
            })
        } else {
            Ok(StorageResult {
                success: true,
                message: "File not found (already deleted)".to_string(),
            })
        }
    }
    
    pub async fn list_keys(&self) -> Result<Vec<String>, StorageError> {
        let mut keys = Vec::new();
        if !self.config.app_data_dir.exists() {
            return Ok(keys);
        }
        
        for entry in fs::read_dir(&self.config.app_data_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            
            if let Some(name_str) = file_name.to_str() {
                let extension = if self.config.encryption_enabled { ".enc" } else { ".json" };
                if name_str.ends_with(extension) {
                    let key = name_str.trim_end_matches(extension);
                    keys.push(key.to_string());
                }
            }
        }
        
        keys.sort();
        Ok(keys)
    }
}

pub fn create_storage_backend(app_handle: AppHandle) -> Result<StorageBackend, StorageError> {
    StorageBackend::new(app_handle)
}

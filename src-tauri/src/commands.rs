use crate::errors::AppError;
use crate::storage::create_storage_backend;
use crate::scheduler::{get_scheduler, WorkSchedule as SchedulerWorkSchedule, SchedulerState};
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

/// Set the user's schedule configuration as a JSON string
#[tauri::command]
pub async fn set_schedule(
    app_handle: AppHandle,
    schedule_json: String,
) -> Result<StorageResult, String> {
    if schedule_json.is_empty() {
        return Err("Schedule JSON cannot be empty".to_string());
    }
    
    // Validate JSON format
    serde_json::from_str::<serde_json::Value>(&schedule_json)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;
    
    let storage = create_storage_backend(app_handle).map_err(|e| format!("Failed to create storage backend: {}", e))?;
    storage.store("user_schedule", &schedule_json).await.map_err(|e| format!("Failed to store schedule: {}", e))
}

/// Get the user's schedule configuration as a JSON string
#[tauri::command]
pub async fn get_schedule(app_handle: AppHandle) -> Result<Option<String>, String> {
    let storage = create_storage_backend(app_handle).map_err(|e| format!("Failed to create storage backend: {}", e))?;
    storage.retrieve("user_schedule").await.map_err(|e| format!("Failed to retrieve schedule: {}", e))
}

// ============================================================================
// SCHEDULER COMMANDS
// ============================================================================

/// Start the backend scheduler with the given schedule
#[tauri::command]
pub async fn start_scheduler(schedule: SchedulerWorkSchedule) -> Result<String, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    
    scheduler.start_scheduler(schedule).await
        .map_err(|e| format!("Failed to start scheduler: {}", e))?;
    
    Ok("Scheduler started successfully".to_string())
}

/// Stop the backend scheduler
#[tauri::command]
pub async fn stop_scheduler() -> Result<String, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    
    scheduler.stop_scheduler().await
        .map_err(|e| format!("Failed to stop scheduler: {}", e))?;
    
    Ok("Scheduler stopped successfully".to_string())
}

/// Get current scheduler state
#[tauri::command]
pub async fn get_scheduler_state() -> Result<SchedulerState, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    Ok(scheduler.get_state())
}

/// Set access token for the scheduler
#[tauri::command]
pub async fn set_scheduler_access_token(access_token: Option<String>) -> Result<String, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    scheduler.set_access_token(access_token);
    Ok("Access token updated".to_string())
}

/// Manual clock-in through backend scheduler
#[tauri::command]
pub async fn scheduler_manual_clock_in() -> Result<bool, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    scheduler.manual_clock_in().await
        .map_err(|e| format!("Manual clock-in failed: {}", e))
}

/// Manual clock-out through backend scheduler
#[tauri::command]
pub async fn scheduler_manual_clock_out(bypass_minimum: Option<bool>) -> Result<bool, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    scheduler.manual_clock_out(bypass_minimum.unwrap_or(false)).await
        .map_err(|e| format!("Manual clock-out failed: {}", e))
}

/// Check if user can clock out (minimum duration check)
#[tauri::command]
pub async fn scheduler_can_clock_out() -> Result<bool, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    Ok(scheduler.can_clock_out())
}

/// Check and perform auto clock-in on app startup
#[tauri::command]
pub async fn scheduler_check_auto_startup() -> Result<bool, String> {
    let scheduler = get_scheduler().ok_or("Scheduler not initialized")?;
    scheduler.check_auto_startup().await
        .map_err(|e| format!("Auto startup check failed: {}", e))
}

/// Initialize background monitoring for sleep/wake detection
#[tauri::command]
pub async fn initialize_background_monitoring(app_handle: AppHandle) -> Result<String, String> {
    // Perform initial auto-startup check
    tokio::spawn(async move {
        // Small delay to ensure everything is initialized
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        println!("Running initial auto-startup check...");
        
        // Get the scheduler instance
        if let Some(scheduler) = get_scheduler() {
            // Try to load access token from storage first
            match retrieve_encrypted_data(app_handle.clone(), "access_token".to_string()).await {
                Ok(Some(token)) => {
                    println!("Loaded access token from storage for auto-startup");
                    scheduler.set_access_token(Some(token.clone()));
                    
                    // Run initial auto-startup check
                    match scheduler.check_auto_startup().await {
                        Ok(clocked_in) => {
                            if clocked_in {
                                println!("Initial auto clock-in completed successfully");
                            } else {
                                println!("Initial auto clock-in skipped (already clocked in or conditions not met)");
                            }
                        }
                        Err(e) => {
                            println!("Initial auto clock-in failed: {:?}", e);
                        }
                    }
                }
                Ok(None) | Err(_) => {
                    println!("No access token found in storage, skipping auto-startup check");
                }
            }
        } else {
            println!("Error: Could not get scheduler instance for auto-startup check");
        }
        
        // Set up gap detection for sleep/wake monitoring
        println!("Starting sleep/wake gap detection monitoring...");
        let mut last_check = std::time::SystemTime::now();
        
        loop {
            // Check every 5 minutes
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            
            let now = std::time::SystemTime::now();
            if let Ok(duration_since_last) = now.duration_since(last_check) {
                // If more than 10 minutes have passed since last check, 
                // it might indicate the system was sleeping
                if duration_since_last.as_secs() > 600 { // 10 minutes
                    println!("Detected potential system wake (gap of {} seconds), checking auto clock-in...", 
                             duration_since_last.as_secs());
                    
                    if let Some(scheduler) = get_scheduler() {
                        match scheduler.check_auto_startup().await {
                            Ok(clocked_in) => {
                                if clocked_in {
                                    println!("Post-wake auto clock-in completed successfully");
                                }
                            }
                            Err(e) => {
                                println!("Post-wake auto clock-in check failed: {:?}", e);
                            }
                        }
                    }
                }
            }
            
            last_check = now;
        }
    });
    
    Ok("Background monitoring initialized".to_string())
}

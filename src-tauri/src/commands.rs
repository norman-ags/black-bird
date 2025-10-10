use crate::errors::AppError;
use crate::storage::create_storage_backend;
use crate::scheduler::{get_scheduler, WorkSchedule as SchedulerWorkSchedule, SchedulerState};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use reqwest;
use chrono;

// EMAPTA API constants
const EMAPTA_TOKEN_ENDPOINT: &str = "https://api.platform.emapta.com/auth/v1/auth/protocol/openid-connect/token";
const EMAPTA_LOGIN_ENDPOINT: &str = "https://api.platform.emapta.com/time-and-attendance/ta/v1/dtr/attendance/login";
const EMAPTA_LOGOUT_ENDPOINT: &str = "https://api.platform.emapta.com/time-and-attendance/ta/v1/dtr/attendance/logout";
const EMAPTA_ATTENDANCE_ENDPOINT: &str = "https://api.platform.emapta.com/time-and-attendance/ta/v1/dtr/attendance";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<i64>,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmaptaApiResponse {
    pub timestamp: String,
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    pub message: String,
    pub result: TokenResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendanceItem {
    pub work_date: String,
    pub attendance_status: String,
    pub date_time_in: Option<String>,
    pub date_time_out: Option<String>,
    pub is_restday: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendanceData {
    pub items: Vec<AttendanceItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendanceApiResponse {
    pub timestamp: String,
    pub status_code: u16,
    pub message: Vec<String>,
    pub data: AttendanceData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub client_id: String,
    pub refresh_token: String,
    pub scope: String,
}

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

// ============================================================================
// BACKEND API CLIENT FUNCTIONS
// ============================================================================

/// Exchange refresh token for access token using EMAPTA API
pub async fn exchange_refresh_token_api(refresh_token: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    
    let request_body = TokenRequest {
        grant_type: "refresh_token".to_string(),
        client_id: "EMAPTA-MYEMAPTAWEB".to_string(),
        refresh_token: refresh_token.to_string(),
        scope: "openid".to_string(),
    };

    let response = client
        .post(EMAPTA_TOKEN_ENDPOINT)
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Token exchange failed: {} - {}", status, error_text));
    }

    // Debug: log the response text before trying to parse it
    let response_text = response.text().await
        .map_err(|e| format!("Failed to get response text: {}", e))?;
    
    println!("EMAPTA API Response: {}", response_text);
    
    let api_response: EmaptaApiResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse API response: {} - Response was: {}", e, response_text))?;

    Ok(api_response.result)
}

/// Perform clock in operation using EMAPTA API
pub async fn clock_in_api(access_token: &str) -> Result<bool, String> {
    println!("[API] Clock-in API called with token: {}", access_token);
    let client = reqwest::Client::new();

    let response = client
        .post(EMAPTA_LOGIN_ENDPOINT)
        .header("application-type", "KEYCLOAK")
        .header("client-code", "EMAPTA-MYEMAPTA")
        .header("authorization", format!("Bearer {}", access_token))
        .header("content-type", "application/json")
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Clock in request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        println!("[API] Clock-in failed with token: {}, status: {}, error: {}", access_token, status, error_text);
        return Err(format!("Clock in failed: {} - {}", status, error_text));
    }

    Ok(true)
}

/// Perform clock out operation using EMAPTA API
pub async fn clock_out_api(access_token: &str) -> Result<bool, String> {
    println!("[API] Clock-out API called with token: {}", access_token);
    let client = reqwest::Client::new();

    let response = client
        .post(EMAPTA_LOGOUT_ENDPOINT)
        .header("application-type", "KEYCLOAK")
        .header("client-code", "EMAPTA-MYEMAPTA")
        .header("authorization", format!("Bearer {}", access_token))
        .header("content-type", "application/json")
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Clock out request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        println!("[API] Clock-out failed with token: {}, status: {}, error: {}", access_token, status, error_text);
        return Err(format!("Clock out failed: {} - {}", status, error_text));
    }

    Ok(true)
}

/// Fetch current attendance status from EMAPTA API
pub async fn get_attendance_status_api(access_token: &str) -> Result<Option<AttendanceItem>, String> {
    println!("[API] Attendance status API called with token: {}", access_token);
    let client = reqwest::Client::new();

    // Get today's date in local timezone
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let response = client
        .get(EMAPTA_ATTENDANCE_ENDPOINT)
        .header("application-type", "KEYCLOAK")
        .header("client-code", "EMAPTA-MYEMAPTA")
        .header("authorization", format!("Bearer {}", access_token))
        .header("content-type", "application/json")
        .query(&[("date_from", &today), ("date_to", &today)])
        .send()
        .await
        .map_err(|e| format!("Attendance status request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        println!("[API] Attendance status failed with token: {}, status: {}, error: {}", access_token, status, error_text);
        return Err(format!("Attendance status failed: {} - {}", status, error_text));
    }

    let attendance_response: AttendanceApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse attendance response: {}", e))?;

    // Find today's attendance record
    Ok(attendance_response.data.items.into_iter()
        .find(|item| item.work_date == today))
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
    println!("[Storage] Frontend requesting retrieval for key: '{}'", key);
    validate_storage_key(&key).map_err(|e| format!("Key validation failed: {}", e))?;
    let storage = create_storage_backend(app_handle).map_err(|e| format!("Failed to create storage backend: {}", e))?;
    let result = storage.retrieve(&key).await.map_err(|e| format!("Retrieval operation failed: {}", e))?;
    println!("[Storage] Retrieved value for key '{}': {}", key, if result.is_some() { "found" } else { "not found" });
    Ok(result)
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

/// Set access token for the scheduler (deprecated - tokens are now managed in storage)
#[tauri::command]
pub async fn set_scheduler_access_token(_access_token: Option<String>) -> Result<String, String> {
    // This function is deprecated as we now use storage-first token management
    // All token management is handled automatically through storage
    Ok("Token management is now storage-first - no action needed".to_string())
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

// ============================================================================
// BACKEND API COMMANDS
// ============================================================================

/// Exchange refresh token for access token and save both (initial setup)
#[tauri::command]
pub async fn api_exchange_refresh_token(
    app_handle: AppHandle,
    refresh_token: String,
) -> Result<TokenResponse, String> {
    // Exchange token via backend API
    let token_response = exchange_refresh_token_api(&refresh_token).await
        .map_err(|e| format!("Token exchange failed: {}", e))?;

    // Save both tokens using shared token manager
    crate::token_manager::save_initial_tokens(
        &app_handle,
        &token_response.refresh_token,
        &token_response.access_token,
    ).await
        .map_err(|e| format!("Failed to save tokens: {}", e))?;

    Ok(token_response)
}

/// Manual clock in operation using shared token logic
#[tauri::command]
pub async fn api_manual_clock_in(app_handle: AppHandle) -> Result<bool, String> {
    crate::token_manager::clock_in_with_shared_tokens(&app_handle).await
        .map_err(|e| format!("Manual clock-in failed: {}", e))
}

/// Manual clock out operation using shared token logic
#[tauri::command]
pub async fn api_manual_clock_out(app_handle: AppHandle) -> Result<bool, String> {
    crate::token_manager::clock_out_with_shared_tokens(&app_handle).await
        .map_err(|e| format!("Manual clock-out failed: {}", e))
}

/// Initialize background monitoring for sleep/wake detection
#[tauri::command]
pub async fn initialize_background_monitoring(app_handle: AppHandle) -> Result<String, String> {
    println!("[Background] Initializing background monitoring and sleep/wake detection...");

    // Perform initial auto-startup check
    tokio::spawn(async move {
        // Small delay to ensure everything is initialized
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        println!("Running initial auto-startup check...");
        
        // Get the scheduler instance
        if let Some(scheduler) = get_scheduler() {
            // Try to load access token from storage first
            // Check if we have tokens in storage and run auto-startup check
            // Scheduler will get tokens from storage using storage-first pattern

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
        } else {
            println!("Error: Could not get scheduler instance for auto-startup check");
        }
        
        // Set up gap detection for sleep/wake monitoring
        println!("Starting sleep/wake gap detection monitoring...");
        let mut last_check = std::time::SystemTime::now();
        
        loop {
            // Check more frequently for better responsiveness
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; // Every 1 minute instead of 5

            let now = std::time::SystemTime::now();

            // Log that monitoring is still active (helps verify tray behavior)
            println!("[Background] Sleep/wake monitoring active - last check was {} seconds ago",
                     now.duration_since(last_check).unwrap_or_default().as_secs());
            if let Ok(duration_since_last) = now.duration_since(last_check) {
                // Lower threshold for wake detection - more sensitive to shorter sleeps
                if duration_since_last.as_secs() > 120 { // 2 minutes instead of 10
                    let gap_seconds = duration_since_last.as_secs();
                    println!("Detected potential system wake (gap of {} seconds), checking auto clock-in...", gap_seconds);

                    // Log wake detection event
                    if let Some(logger) = crate::logging::get_logger() {
                        let _ = logger.log_wake_detected(gap_seconds).await;
                    }

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

/// Get current attendance status using shared token logic
#[tauri::command]
pub async fn api_get_attendance_status(app_handle: AppHandle) -> Result<Option<AttendanceItem>, String> {
    crate::token_manager::attendance_check_with_shared_tokens(&app_handle).await
        .map_err(|e| format!("Attendance status check failed: {}", e))
}

/// Setup both refresh and access tokens with validation (Phase 3 enhancement)
#[tauri::command]
pub async fn api_setup_dual_tokens(
    app_handle: AppHandle,
    refresh_token: String,
    access_token: String,
) -> Result<String, String> {
    // Validate token format
    if refresh_token.trim().is_empty() {
        return Err("Refresh token cannot be empty".to_string());
    }

    if access_token.trim().is_empty() {
        return Err("Access token cannot be empty".to_string());
    }

    // Basic token format validation
    if refresh_token.len() < 10 {
        return Err("Refresh token appears too short - please check the token".to_string());
    }

    if access_token.len() < 10 {
        return Err("Access token appears too short - please check the token".to_string());
    }

    // Save both tokens using the shared token manager
    crate::token_manager::save_initial_tokens(
        &app_handle,
        &refresh_token,
        &access_token,
    ).await
        .map_err(|e| format!("Failed to save tokens: {}", e))?;

    // Validate tokens by trying an API call
    match crate::token_manager::attendance_check_with_shared_tokens(&app_handle).await {
        Ok(_) => {
            // Tokens are valid - setup complete
            Ok("Tokens validated and saved successfully! Setup complete.".to_string())
        }
        Err(e) => {
            // Token validation failed - provide clear error message
            let error_msg = format!("Token validation failed: {}. Please check your tokens and try again.", e);
            Err(error_msg)
        }
    }
}

// ============================================================================
// AUTOSTART COMMANDS (Phase 3 Enhancement)
// ============================================================================

/// Enable auto-launch on system startup
#[tauri::command]
pub async fn enable_autostart(app_handle: AppHandle) -> Result<String, String> {
    use tauri_plugin_autostart::ManagerExt;

    match app_handle.autolaunch().enable() {
        Ok(_) => Ok("Auto-launch enabled successfully".to_string()),
        Err(e) => Err(format!("Failed to enable auto-launch: {}", e))
    }
}

/// Disable auto-launch on system startup
#[tauri::command]
pub async fn disable_autostart(app_handle: AppHandle) -> Result<String, String> {
    use tauri_plugin_autostart::ManagerExt;

    match app_handle.autolaunch().disable() {
        Ok(_) => Ok("Auto-launch disabled successfully".to_string()),
        Err(e) => Err(format!("Failed to disable auto-launch: {}", e))
    }
}

/// Check if auto-launch is currently enabled
#[tauri::command]
pub async fn is_autostart_enabled(app_handle: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;

    match app_handle.autolaunch().is_enabled() {
        Ok(enabled) => Ok(enabled),
        Err(e) => Err(format!("Failed to check auto-launch status: {}", e))
    }
}

// ============================================================================
// ACTIVITY LOGGING COMMANDS (Phase 4 Feature)
// ============================================================================

/// Get recent activity log entries
#[tauri::command]
pub async fn get_activity_logs(limit: Option<usize>) -> Result<Vec<crate::logging::LogEntry>, String> {
    let logger = crate::logging::get_logger().ok_or("Activity logger not initialized")?;
    logger.get_recent_entries(limit).await
        .map_err(|e| format!("Failed to get activity logs: {}", e))
}

/// Get filtered activity log entries
#[tauri::command]
pub async fn get_filtered_activity_logs(
    action_filter: Option<String>,
    status_filter: Option<String>,
    limit: Option<usize>
) -> Result<Vec<crate::logging::LogEntry>, String> {
    let logger = crate::logging::get_logger().ok_or("Activity logger not initialized")?;

    // Parse filter strings to enums
    let action = match action_filter.as_deref() {
        Some("clock_in") => Some(crate::logging::LogAction::ClockIn),
        Some("clock_out") => Some(crate::logging::LogAction::ClockOut),
        Some("attendance_check") => Some(crate::logging::LogAction::AttendanceCheck),
        Some("token_refresh") => Some(crate::logging::LogAction::TokenRefresh),
        Some("wake_detected") => Some(crate::logging::LogAction::WakeDetected),
        Some("schedule_updated") => Some(crate::logging::LogAction::ScheduleUpdated),
        Some("app_startup") => Some(crate::logging::LogAction::AppStartup),
        Some("error") => Some(crate::logging::LogAction::Error),
        _ => None,
    };

    let status = match status_filter.as_deref() {
        Some("success") => Some(crate::logging::LogStatus::Success),
        Some("failed") => Some(crate::logging::LogStatus::Failed),
        Some("warning") => Some(crate::logging::LogStatus::Warning),
        Some("info") => Some(crate::logging::LogStatus::Info),
        _ => None,
    };

    logger.get_filtered_entries(action, status, limit).await
        .map_err(|e| format!("Failed to get filtered activity logs: {}", e))
}

/// Clear all activity logs
#[tauri::command]
pub async fn clear_activity_logs() -> Result<u32, String> {
    let logger = crate::logging::get_logger().ok_or("Activity logger not initialized")?;
    logger.clear_all_logs().await
        .map_err(|e| format!("Failed to clear activity logs: {}", e))
}

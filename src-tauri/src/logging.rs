use chrono::{DateTime, Utc, TimeZone, Datelike};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use crate::errors::AppError;
use crate::storage::create_storage_backend;

/// Log entry representing a single app operation or event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Unique identifier for this log entry
    pub id: String,
    /// Timestamp when the event occurred (ISO 8601 UTC)
    pub timestamp: String,
    /// Type of action or operation
    pub action: LogAction,
    /// Status/result of the operation
    pub status: LogStatus,
    /// Human-readable description of what happened
    pub details: String,
    /// Additional structured metadata
    pub metadata: LogMetadata,
}

/// Types of actions that can be logged
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogAction {
    ClockIn,
    ClockOut,
    AttendanceCheck,
    TokenRefresh,
    WakeDetected,
    ScheduleUpdated,
    AppStartup,
    Error,
}

/// Status/result of the logged operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogStatus {
    Success,
    Failed,
    Warning,
    Info,
}

/// Additional metadata for log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogMetadata {
    /// Duration of the operation in milliseconds (optional)
    pub duration: Option<u64>,
    /// Type of trigger that initiated this operation
    pub trigger_type: Option<String>,
    /// API endpoint that was called (if applicable)
    pub api_endpoint: Option<String>,
    /// Error code or additional error details (optional)
    pub error_code: Option<String>,
}

/// Monthly log container with auto-cleanup
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyLogContainer {
    /// Year and month (e.g., "2025_10")
    pub month_key: String,
    /// All log entries for this month
    pub entries: Vec<LogEntry>,
    /// Total number of entries ever added (for cleanup tracking)
    pub total_entries: usize,
    /// Timestamp when this container was created
    pub created_at: String,
    /// Timestamp when this container was last updated
    pub updated_at: String,
}

/// Logger service for managing structured activity logs
pub struct ActivityLogger {
    app_handle: AppHandle,
}

impl ActivityLogger {
    /// Create a new activity logger instance
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Log a new activity entry
    pub async fn log(&self, action: LogAction, status: LogStatus, details: String, metadata: LogMetadata) -> Result<(), AppError> {
        let now = Utc::now();
        let log_id = format!("log_{}_{:03}", now.timestamp(), now.timestamp_subsec_millis() % 1000);

        let entry = LogEntry {
            id: log_id,
            timestamp: now.to_rfc3339(),
            action,
            status,
            details,
            metadata,
        };

        self.add_entry_to_monthly_log(entry).await
    }

    /// Convenience method to log clock-in operations
    pub async fn log_clock_in(&self, success: bool, trigger_type: &str, duration_ms: Option<u64>, error: Option<&str>) -> Result<(), AppError> {
        let status = if success { LogStatus::Success } else { LogStatus::Failed };
        let details = if success {
            format!("Clock-in completed successfully (trigger: {})", trigger_type)
        } else {
            format!("Clock-in failed (trigger: {}): {}", trigger_type, error.unwrap_or("Unknown error"))
        };

        let metadata = LogMetadata {
            duration: duration_ms,
            trigger_type: Some(trigger_type.to_string()),
            api_endpoint: Some("/dtr/attendance/login".to_string()),
            error_code: error.map(|e| e.to_string()),
        };

        self.log(LogAction::ClockIn, status, details, metadata).await
    }

    /// Convenience method to log clock-out operations
    pub async fn log_clock_out(&self, success: bool, trigger_type: &str, duration_ms: Option<u64>, error: Option<&str>) -> Result<(), AppError> {
        let status = if success { LogStatus::Success } else { LogStatus::Failed };
        let details = if success {
            format!("Clock-out completed successfully (trigger: {})", trigger_type)
        } else {
            format!("Clock-out failed (trigger: {}): {}", trigger_type, error.unwrap_or("Unknown error"))
        };

        let metadata = LogMetadata {
            duration: duration_ms,
            trigger_type: Some(trigger_type.to_string()),
            api_endpoint: Some("/dtr/attendance/logout".to_string()),
            error_code: error.map(|e| e.to_string()),
        };

        self.log(LogAction::ClockOut, status, details, metadata).await
    }

    /// Convenience method to log attendance checks
    pub async fn log_attendance_check(&self, success: bool, duration_ms: Option<u64>, error: Option<&str>) -> Result<(), AppError> {
        let status = if success { LogStatus::Success } else { LogStatus::Failed };
        let details = if success {
            "Attendance status check completed successfully".to_string()
        } else {
            format!("Attendance status check failed: {}", error.unwrap_or("Unknown error"))
        };

        let metadata = LogMetadata {
            duration: duration_ms,
            trigger_type: Some("api_check".to_string()),
            api_endpoint: Some("/dtr/attendance".to_string()),
            error_code: error.map(|e| e.to_string()),
        };

        self.log(LogAction::AttendanceCheck, status, details, metadata).await
    }

    /// Convenience method to log token refresh operations
    pub async fn log_token_refresh(&self, success: bool, duration_ms: Option<u64>, error: Option<&str>) -> Result<(), AppError> {
        let status = if success { LogStatus::Success } else { LogStatus::Failed };
        let details = if success {
            "Token refresh completed successfully".to_string()
        } else {
            format!("Token refresh failed: {}", error.unwrap_or("Unknown error"))
        };

        let metadata = LogMetadata {
            duration: duration_ms,
            trigger_type: Some("auto_refresh".to_string()),
            api_endpoint: Some("/auth/v1/auth/protocol/openid-connect/token".to_string()),
            error_code: error.map(|e| e.to_string()),
        };

        self.log(LogAction::TokenRefresh, status, details, metadata).await
    }

    /// Convenience method to log wake detection events
    pub async fn log_wake_detected(&self, gap_seconds: u64) -> Result<(), AppError> {
        let details = format!("System wake detected after {} seconds of inactivity", gap_seconds);

        let metadata = LogMetadata {
            duration: Some(gap_seconds * 1000), // Convert to milliseconds for consistency
            trigger_type: Some("wake_detection".to_string()),
            api_endpoint: None,
            error_code: None,
        };

        self.log(LogAction::WakeDetected, LogStatus::Info, details, metadata).await
    }

    /// Convenience method to log app startup events
    pub async fn log_app_startup(&self, auto_clock_in_attempted: bool, auto_clock_in_success: Option<bool>) -> Result<(), AppError> {
        let details = if auto_clock_in_attempted {
            match auto_clock_in_success {
                Some(true) => "App startup completed with successful auto clock-in".to_string(),
                Some(false) => "App startup completed with failed auto clock-in".to_string(),
                None => "App startup completed with auto clock-in attempt (result pending)".to_string(),
            }
        } else {
            "App startup completed (no auto clock-in needed)".to_string()
        };

        let metadata = LogMetadata {
            duration: None,
            trigger_type: Some("app_startup".to_string()),
            api_endpoint: None,
            error_code: None,
        };

        let status = if auto_clock_in_attempted && auto_clock_in_success == Some(false) {
            LogStatus::Warning
        } else {
            LogStatus::Success
        };

        self.log(LogAction::AppStartup, status, details, metadata).await
    }

    /// Get recent log entries (up to limit, defaulting to 100)
    pub async fn get_recent_entries(&self, limit: Option<usize>) -> Result<Vec<LogEntry>, AppError> {
        let limit = limit.unwrap_or(100);
        let current_month = get_current_month_key();

        // Get current month's logs
        let mut all_entries = Vec::new();

        if let Ok(Some(current_container)) = self.get_monthly_container(&current_month).await {
            // Get most recent entries from current month
            let mut current_entries = current_container.entries;
            current_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Most recent first
            all_entries.extend(current_entries);
        }

        // If we need more entries, get from previous month
        if all_entries.len() < limit {
            let prev_month = get_previous_month_key(&current_month);
            if let Ok(Some(prev_container)) = self.get_monthly_container(&prev_month).await {
                let mut prev_entries = prev_container.entries;
                prev_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Most recent first
                all_entries.extend(prev_entries);
            }
        }

        // Sort all entries by timestamp (most recent first) and limit
        all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all_entries.truncate(limit);

        Ok(all_entries)
    }

    /// Get log entries filtered by action and/or status
    pub async fn get_filtered_entries(&self, action_filter: Option<LogAction>, status_filter: Option<LogStatus>, limit: Option<usize>) -> Result<Vec<LogEntry>, AppError> {
        let all_entries = self.get_recent_entries(limit).await?;

        let filtered = all_entries.into_iter()
            .filter(|entry| {
                if let Some(ref action) = action_filter {
                    if std::mem::discriminant(&entry.action) != std::mem::discriminant(action) {
                        return false;
                    }
                }
                if let Some(ref status) = status_filter {
                    if std::mem::discriminant(&entry.status) != std::mem::discriminant(status) {
                        return false;
                    }
                }
                true
            })
            .collect();

        Ok(filtered)
    }

    /// Clear all log entries (for maintenance)
    pub async fn clear_all_logs(&self) -> Result<u32, AppError> {
        let storage = create_storage_backend(self.app_handle.clone())?;
        let keys = storage.list_keys().await?;

        let mut deleted_count = 0;

        for key in keys {
            if key.starts_with("logs_") {
                storage.delete(&key).await?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Add entry to monthly log container with auto-cleanup
    async fn add_entry_to_monthly_log(&self, entry: LogEntry) -> Result<(), AppError> {
        let month_key = get_current_month_key();
        let storage_key = format!("logs_{}", month_key);

        // Get or create monthly container
        let mut container = match self.get_monthly_container(&month_key).await? {
            Some(container) => container,
            None => MonthlyLogContainer {
                month_key: month_key.clone(),
                entries: Vec::new(),
                total_entries: 0,
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
            }
        };

        // Add new entry
        container.entries.push(entry);
        container.total_entries += 1;
        container.updated_at = Utc::now().to_rfc3339();

        // Auto-cleanup: keep only the most recent 1000 entries per month
        if container.entries.len() > 1000 {
            // Sort by timestamp (most recent first) and keep only 1000
            container.entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            container.entries.truncate(1000);
        }

        // Save updated container
        let storage = create_storage_backend(self.app_handle.clone())?;
        let container_json = serde_json::to_string(&container)
            .map_err(|e| AppError::storage(&format!("Failed to serialize log container: {}", e)))?;

        storage.store(&storage_key, &container_json).await?;

        // Clean up old months (keep only 6 months)
        self.cleanup_old_months().await?;

        Ok(())
    }

    /// Get monthly log container from storage
    async fn get_monthly_container(&self, month_key: &str) -> Result<Option<MonthlyLogContainer>, AppError> {
        let storage_key = format!("logs_{}", month_key);
        let storage = create_storage_backend(self.app_handle.clone())?;

        if let Some(container_json) = storage.retrieve(&storage_key).await? {
            let container: MonthlyLogContainer = serde_json::from_str(&container_json)
                .map_err(|e| AppError::storage(&format!("Failed to deserialize log container: {}", e)))?;
            Ok(Some(container))
        } else {
            Ok(None)
        }
    }

    /// Clean up log containers older than 6 months
    async fn cleanup_old_months(&self) -> Result<(), AppError> {
        let storage = create_storage_backend(self.app_handle.clone())?;
        let keys = storage.list_keys().await?;

        let now = Utc::now();
        let six_months_ago = now - chrono::Duration::days(6 * 30); // Approximate 6 months

        for key in keys {
            if key.starts_with("logs_") {
                let month_key = &key[5..]; // Remove "logs_" prefix

                if let Some(month_date) = parse_month_key(month_key) {
                    if month_date < six_months_ago {
                        println!("[Logging] Cleaning up old log container: {}", key);
                        storage.delete(&key).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Get current month key in format "YYYY_MM"
fn get_current_month_key() -> String {
    let now = Utc::now();
    format!("{}_{:02}", now.year(), now.month())
}

/// Get previous month key from a given month key
fn get_previous_month_key(current_month: &str) -> String {
    if let Some((year, month)) = parse_month_key_parts(current_month) {
        if month == 1 {
            format!("{}_{:02}", year - 1, 12)
        } else {
            format!("{}_{:02}", year, month - 1)
        }
    } else {
        // Fallback to current month if parsing fails
        get_current_month_key()
    }
}

/// Parse month key into year and month numbers
fn parse_month_key_parts(month_key: &str) -> Option<(i32, u32)> {
    let parts: Vec<&str> = month_key.split('_').collect();
    if parts.len() == 2 {
        if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
            return Some((year, month));
        }
    }
    None
}

/// Parse month key into a DateTime for comparison
fn parse_month_key(month_key: &str) -> Option<DateTime<Utc>> {
    if let Some((year, month)) = parse_month_key_parts(month_key) {
        Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).single()
    } else {
        None
    }
}

// Global logger instance
static mut LOGGER: Option<ActivityLogger> = None;
static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global activity logger
pub fn initialize_logger(app_handle: AppHandle) {
    unsafe {
        LOGGER_INIT.call_once(|| {
            LOGGER = Some(ActivityLogger::new(app_handle));
        });
    }
}

/// Force re-initialize the global activity logger (for debugging)
pub fn force_reinitialize_logger(app_handle: AppHandle) {
    unsafe {
        println!("[LOGGING] Force re-initializing logger...");
        LOGGER = Some(ActivityLogger::new(app_handle));
        println!("[LOGGING] Logger force re-initialized");
    }
}

/// Get the global activity logger instance
pub fn get_logger() -> Option<&'static ActivityLogger> {
    unsafe { LOGGER.as_ref() }
}
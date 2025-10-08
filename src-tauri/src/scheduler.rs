use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::errors::AppError;

/// Work schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSchedule {
    pub auto_schedule_enabled: bool,
    pub clock_in_time: String, // HH:MM format
    pub timezone: String,
    pub min_work_duration_minutes: u32,
}

/// Scheduler operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    ClockIn,
    ClockOut,
}

/// Scheduled operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledOperation {
    pub id: String,
    pub operation_type: OperationType,
    pub scheduled_time: String, // ISO 8601
    pub status: String,         // pending, completed, failed, cancelled
    pub actual_time: Option<String>,
    pub error_message: Option<String>,
}

/// Current session state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub clocked_in: bool,
    pub clock_in_time: Option<String>,
    pub expected_clock_out_time: Option<String>,
}

/// Scheduler state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerState {
    pub is_running: bool,
    pub current_session: SessionState,
    pub pending_operations: Vec<ScheduledOperation>,
    pub last_error: Option<String>,
}

/// Scheduler events sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SchedulerEvent {
    #[serde(rename = "scheduler_started")]
    SchedulerStarted,
    #[serde(rename = "scheduler_stopped")]
    SchedulerStopped,
    #[serde(rename = "clock_in_scheduled")]
    ClockInScheduled { operation_id: String, scheduled_time: String },
    #[serde(rename = "clock_out_scheduled")]
    ClockOutScheduled { operation_id: String, scheduled_time: String },
    #[serde(rename = "clock_in_succeeded")]
    ClockInSucceeded { operation_id: String, actual_time: String },
    #[serde(rename = "clock_out_succeeded")]
    ClockOutSucceeded { operation_id: String, actual_time: String },
    #[serde(rename = "clock_in_failed")]
    ClockInFailed { operation_id: String, error: String },
    #[serde(rename = "clock_out_failed")]
    ClockOutFailed { operation_id: String, error: String },
    #[serde(rename = "schedule_updated")]
    ScheduleUpdated { schedule: WorkSchedule },
    #[serde(rename = "state_changed")]
    StateChanged { state: SchedulerState },
    #[serde(rename = "auto_startup_completed")]
    AutoStartupCompleted { success: bool },
}

/// Internal scheduler task handle
type TaskHandle = tokio::task::JoinHandle<()>;

/// Backend scheduler for automatic clock operations
pub struct BackendScheduler {
    app_handle: AppHandle,
    state: Arc<Mutex<SchedulerState>>,
    schedule: Arc<Mutex<Option<WorkSchedule>>>,
    task_handles: Arc<Mutex<HashMap<String, TaskHandle>>>,
    access_token: Arc<Mutex<Option<String>>>,
}

impl BackendScheduler {
    /// Create a new backend scheduler
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            state: Arc::new(Mutex::new(SchedulerState {
                is_running: false,
                current_session: SessionState {
                    clocked_in: false,
                    clock_in_time: None,
                    expected_clock_out_time: None,
                },
                pending_operations: Vec::new(),
                last_error: None,
            })),
            schedule: Arc::new(Mutex::new(None)),
            task_handles: Arc::new(Mutex::new(HashMap::new())),
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the access token for API calls
    pub fn set_access_token(&self, token: Option<String>) {
        let mut access_token = self.access_token.lock().unwrap();
        *access_token = token;
    }

    /// Check and perform auto clock-in on app startup
    pub async fn check_auto_startup(&self) -> Result<bool, AppError> {
        println!("[Scheduler] Checking if auto clock-in should run...");
        
        // Check if we have an access token
        let token = {
            let access_token = self.access_token.lock().unwrap();
            access_token.clone()
        };

        if token.is_none() {
            println!("[Scheduler] No access token available, skipping auto clock-in");
            return Ok(false);
        }

        // Check if already clocked in
        let current_state = {
            let state = self.state.lock().unwrap();
            state.current_session.clone()
        };

        if current_state.clocked_in {
            println!("[Scheduler] Already clocked in, skipping auto clock-in");
            return Ok(false);
        }

        // Check if already clocked in today (by checking clock-in time)
        let today = chrono::Local::now().date_naive();
        if let Some(clock_in_time) = current_state.clock_in_time {
            if let Ok(last_clock_in) = chrono::DateTime::parse_from_rfc3339(&clock_in_time) {
                let last_clock_in_date = last_clock_in.with_timezone(&chrono::Local).date_naive();
                if last_clock_in_date == today {
                    println!("[Scheduler] Already clocked in today ({}), skipping auto clock-in", today);
                    return Ok(false);
                } else {
                    println!("[Scheduler] Last clock-in was on {}, today is {} - proceeding with auto clock-in", 
                            last_clock_in_date, today);
                }
            }
        } else {
            println!("[Scheduler] No previous clock-in time found - proceeding with auto clock-in");
        }

        // TODO: Add attendance API check for rest days/leave
        // For now, proceed with auto clock-in
        
        println!("[Scheduler] Conditions met, attempting auto clock-in...");
        
        // Attempt auto clock-in using the manual clock-in function
        match self.manual_clock_in().await {
            Ok(success) => {
                if success {
                    println!("[Scheduler] Auto clock-in successful!");
                    // Emit a specific auto-startup event
                    let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::AutoStartupCompleted { success: true });
                } else {
                    println!("[Scheduler] Auto clock-in failed");
                    let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::AutoStartupCompleted { success: false });
                }
                Ok(success)
            }
            Err(error) => {
                println!("[Scheduler] Error during auto clock-in: {:?}", error);
                let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::AutoStartupCompleted { success: false });
                Err(error)
            }
        }
    }

    /// Start the scheduler with the given schedule
    pub async fn start_scheduler(&self, schedule: WorkSchedule) -> Result<(), AppError> {
        println!("[Scheduler] Starting with schedule: {:?}", schedule);
        
        // Update schedule
        {
            let mut sched = self.schedule.lock().unwrap();
            *sched = Some(schedule.clone());
        }

        // Update state
        {
            let mut state = self.state.lock().unwrap();
            state.is_running = true;
            state.last_error = None;
        }

        // Clear existing tasks
        self.cancel_all_tasks().await;

        // Emit event
        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::SchedulerStarted);
        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ScheduleUpdated { schedule: schedule.clone() });

        // Schedule operations if auto scheduling is enabled
        if schedule.auto_schedule_enabled {
            self.schedule_next_clock_in().await?;
        }

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop_scheduler(&self) -> Result<(), AppError> {
        println!("[Scheduler] Stopping scheduler");
        
        // Update state
        {
            let mut state = self.state.lock().unwrap();
            state.is_running = false;
            state.pending_operations.clear();
        }

        // Cancel all tasks
        self.cancel_all_tasks().await;

        // Emit event
        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::SchedulerStopped);

        Ok(())
    }

    /// Get current scheduler state
    pub fn get_state(&self) -> SchedulerState {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    /// Manual clock in
    pub async fn manual_clock_in(&self) -> Result<bool, AppError> {
        println!("[Scheduler] Manual clock in requested");
        
        let token = {
            let access_token = self.access_token.lock().unwrap();
            access_token.clone()
        };

        if token.is_none() {
            return Err(AppError::authentication("No access token available".to_string()));
        }

        // Simulate API call (replace with actual API call)
        let success = self.call_clock_in_api(&token.unwrap()).await?;
        
        if success {
            let now = chrono::Utc::now().to_rfc3339();
            let expected_clock_out = self.calculate_expected_clock_out_time(&now);
            
            // Update state
            {
                let mut state = self.state.lock().unwrap();
                state.current_session.clocked_in = true;
                state.current_session.clock_in_time = Some(now.clone());
                state.current_session.expected_clock_out_time = Some(expected_clock_out);
                
                // Cancel pending clock-in operations
                state.pending_operations.retain(|op| {
                    !matches!(op.operation_type, OperationType::ClockIn) || op.status != "pending"
                });
            }

            // Schedule clock out
            self.schedule_clock_out().await?;
            
            // Emit event
            let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockInSucceeded {
                operation_id: "manual".to_string(),
                actual_time: now,
            });
        }

        Ok(success)
    }

    /// Manual clock out
    pub async fn manual_clock_out(&self, bypass_minimum: bool) -> Result<bool, AppError> {
        println!("[Scheduler] Manual clock out requested (bypass_minimum: {})", bypass_minimum);
        
        if !bypass_minimum && !self.can_clock_out() {
            return Err(AppError::validation("operation", "Cannot clock out before minimum work duration"));
        }

        let token = {
            let access_token = self.access_token.lock().unwrap();
            access_token.clone()
        };

        if token.is_none() {
            return Err(AppError::authentication("No access token available".to_string()));
        }

        // Simulate API call (replace with actual API call)
        let success = self.call_clock_out_api(&token.unwrap()).await?;
        
        if success {
            let now = chrono::Utc::now().to_rfc3339();
            
            // Update state
            {
                let mut state = self.state.lock().unwrap();
                state.current_session.clocked_in = false;
                state.current_session.clock_in_time = None;
                state.current_session.expected_clock_out_time = None;
                
                // Cancel pending clock-out operations
                state.pending_operations.retain(|op| {
                    !matches!(op.operation_type, OperationType::ClockOut) || op.status != "pending"
                });
            }

            // Schedule next clock in
            self.schedule_next_clock_in().await?;
            
            // Emit event
            let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutSucceeded {
                operation_id: "manual".to_string(),
                actual_time: now,
            });
        }

        Ok(success)
    }

    /// Check if user can clock out (minimum duration check)
    pub fn can_clock_out(&self) -> bool {
        let state = self.state.lock().unwrap();
        let schedule = self.schedule.lock().unwrap();
        
        if let (Some(clock_in_time), Some(schedule)) = (&state.current_session.clock_in_time, &*schedule) {
            if let Ok(clock_in_dt) = DateTime::parse_from_rfc3339(clock_in_time) {
                let now = chrono::Utc::now();
                let elapsed_minutes = (now - clock_in_dt.with_timezone(&chrono::Utc)).num_minutes() as u32;
                return elapsed_minutes >= schedule.min_work_duration_minutes;
            }
        }
        
        false
    }

    /// Schedule the next clock-in operation
    async fn schedule_next_clock_in(&self) -> Result<(), AppError> {
        let schedule = {
            let schedule = self.schedule.lock().unwrap();
            schedule.clone()
        };

        let Some(schedule) = schedule else {
            return Ok(());
        };

        if !schedule.auto_schedule_enabled {
            return Ok(());
        }

        let next_clock_in_time = self.get_next_clock_in_time(&schedule)?;
        let operation_id = format!("clock_in_{}", next_clock_in_time.timestamp());
        
        // Add to pending operations
        {
            let mut state = self.state.lock().unwrap();
            state.pending_operations.push(ScheduledOperation {
                id: operation_id.clone(),
                operation_type: OperationType::ClockIn,
                scheduled_time: next_clock_in_time.to_rfc3339(),
                status: "pending".to_string(),
                actual_time: None,
                error_message: None,
            });
        }

        // Create shared state for the async task
        let app_handle = self.app_handle.clone();
        let state = Arc::clone(&self.state);
        let access_token = Arc::clone(&self.access_token);
        let schedule_ref = Arc::clone(&self.schedule);
        let operation_id_clone = operation_id.clone();
        
        let delay = (next_clock_in_time.timestamp() - chrono::Utc::now().timestamp()) as u64;
        let delay_duration = Duration::from_secs(delay.max(1)); // Minimum 1 second delay
        
        let task = tokio::spawn(async move {
            sleep(delay_duration).await;
            
            // Execute clock in
            let _ = execute_scheduled_clock_in(
                app_handle,
                state,
                access_token,
                schedule_ref,
                &operation_id_clone
            ).await;
        });

        // Store task handle
        {
            let mut handles = self.task_handles.lock().unwrap();
            handles.insert(operation_id.clone(), task);
        }

        // Emit event
        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockInScheduled {
            operation_id,
            scheduled_time: next_clock_in_time.to_rfc3339(),
        });

        Ok(())
    }

    /// Schedule clock-out operation
    async fn schedule_clock_out(&self) -> Result<(), AppError> {
        let (clock_in_time, schedule) = {
            let state = self.state.lock().unwrap();
            let schedule = self.schedule.lock().unwrap();
            (state.current_session.clock_in_time.clone(), schedule.clone())
        };

        let (Some(clock_in_time), Some(_schedule)) = (clock_in_time, schedule) else {
            return Ok(());
        };

        let clock_out_time = self.calculate_expected_clock_out_time(&clock_in_time);
        let clock_out_dt = DateTime::parse_from_rfc3339(&clock_out_time)
            .map_err(|_| AppError::validation("time", "Invalid clock out time"))?;
        
        let operation_id = format!("clock_out_{}", clock_out_dt.timestamp());
        
        // Add to pending operations
        {
            let mut state = self.state.lock().unwrap();
            state.pending_operations.push(ScheduledOperation {
                id: operation_id.clone(),
                operation_type: OperationType::ClockOut,
                scheduled_time: clock_out_time.clone(),
                status: "pending".to_string(),
                actual_time: None,
                error_message: None,
            });
        }

        // Create shared state for the async task
        let app_handle = self.app_handle.clone();
        let state = Arc::clone(&self.state);
        let access_token = Arc::clone(&self.access_token);
        let schedule_ref = Arc::clone(&self.schedule);
        let operation_id_clone = operation_id.clone();
        
        let delay = (clock_out_dt.timestamp() - chrono::Utc::now().timestamp()) as u64;
        let delay_duration = Duration::from_secs(delay.max(1)); // Minimum 1 second delay
        
        let task = tokio::spawn(async move {
            sleep(delay_duration).await;
            
            // Execute clock out
            let _ = execute_scheduled_clock_out(
                app_handle,
                state,
                access_token,
                schedule_ref,
                &operation_id_clone
            ).await;
        });

        // Store task handle
        {
            let mut handles = self.task_handles.lock().unwrap();
            handles.insert(operation_id.clone(), task);
        }

        // Emit event
        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutScheduled {
            operation_id,
            scheduled_time: clock_out_time,
        });

        Ok(())
    }

    /// Execute automatic clock-in
    async fn execute_clock_in(&self, operation_id: &str) -> Result<(), AppError> {
        println!("[Scheduler] Executing automatic clock-in: {}", operation_id);
        
        let token = {
            let access_token = self.access_token.lock().unwrap();
            access_token.clone()
        };

        let result = if let Some(token) = token {
            self.call_clock_in_api(&token).await
        } else {
            Err(AppError::authentication("No access token available".to_string()))
        };

        let now = chrono::Utc::now().to_rfc3339();

        // Update operation status
        {
            let mut state = self.state.lock().unwrap();
            if let Some(operation) = state.pending_operations.iter_mut().find(|op| op.id == operation_id) {
                operation.actual_time = Some(now.clone());
                
                match result {
                    Ok(true) => {
                        operation.status = "completed".to_string();
                        
                        // Update session state
                        let expected_clock_out = self.calculate_expected_clock_out_time(&now);
                        state.current_session.clocked_in = true;
                        state.current_session.clock_in_time = Some(now.clone());
                        state.current_session.expected_clock_out_time = Some(expected_clock_out);
                        
                        // Emit success event
                        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockInSucceeded {
                            operation_id: operation_id.to_string(),
                            actual_time: now,
                        });
                        
                        // Schedule clock out
                        let _ = self.schedule_clock_out().await;
                    }
                    Ok(false) => {
                        operation.status = "failed".to_string();
                        operation.error_message = Some("Clock-in API returned false".to_string());
                        
                        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockInFailed {
                            operation_id: operation_id.to_string(),
                            error: "API returned false".to_string(),
                        });
                    }
                    Err(err) => {
                        operation.status = "failed".to_string();
                        operation.error_message = Some(err.to_string());
                        
                        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockInFailed {
                            operation_id: operation_id.to_string(),
                            error: err.to_string(),
                        });
                    }
                }
            }
        }

        // Remove task handle
        {
            let mut handles = self.task_handles.lock().unwrap();
            handles.remove(operation_id);
        }

        Ok(())
    }

    /// Execute automatic clock-out
    async fn execute_clock_out(&self, operation_id: &str) -> Result<(), AppError> {
        println!("[Scheduler] Executing automatic clock-out: {}", operation_id);
        
        let token = {
            let access_token = self.access_token.lock().unwrap();
            access_token.clone()
        };

        let result = if let Some(token) = token {
            self.call_clock_out_api(&token).await
        } else {
            Err(AppError::authentication("No access token available".to_string()))
        };

        let now = chrono::Utc::now().to_rfc3339();

        // Update operation status
        {
            let mut state = self.state.lock().unwrap();
            if let Some(operation) = state.pending_operations.iter_mut().find(|op| op.id == operation_id) {
                operation.actual_time = Some(now.clone());
                
                match result {
                    Ok(true) => {
                        operation.status = "completed".to_string();
                        
                        // Update session state
                        state.current_session.clocked_in = false;
                        state.current_session.clock_in_time = None;
                        state.current_session.expected_clock_out_time = None;
                        
                        // Emit success event
                        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutSucceeded {
                            operation_id: operation_id.to_string(),
                            actual_time: now,
                        });
                        
                        // Schedule next clock in
                        let _ = self.schedule_next_clock_in().await;
                    }
                    Ok(false) => {
                        operation.status = "failed".to_string();
                        operation.error_message = Some("Clock-out API returned false".to_string());
                        
                        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutFailed {
                            operation_id: operation_id.to_string(),
                            error: "API returned false".to_string(),
                        });
                    }
                    Err(err) => {
                        operation.status = "failed".to_string();
                        operation.error_message = Some(err.to_string());
                        
                        let _ = self.app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutFailed {
                            operation_id: operation_id.to_string(),
                            error: err.to_string(),
                        });
                    }
                }
            }
        }

        // Remove task handle
        {
            let mut handles = self.task_handles.lock().unwrap();
            handles.remove(operation_id);
        }

        Ok(())
    }

    /// Cancel all scheduled tasks
    async fn cancel_all_tasks(&self) {
        let mut handles = self.task_handles.lock().unwrap();
        for (_, handle) in handles.drain() {
            handle.abort();
        }
    }

    /// Get next clock-in time based on schedule
    fn get_next_clock_in_time(&self, schedule: &WorkSchedule) -> Result<DateTime<chrono::Utc>, AppError> {
        let now = Local::now();
        let time_parts: Vec<&str> = schedule.clock_in_time.split(':').collect();
        
        if time_parts.len() != 2 {
            return Err(AppError::validation("schedule", "Invalid clock-in time format"));
        }
        
        let hour: u32 = time_parts[0].parse()
            .map_err(|_| AppError::validation("schedule", "Invalid hour in clock-in time"))?;
        let minute: u32 = time_parts[1].parse()
            .map_err(|_| AppError::validation("schedule", "Invalid minute in clock-in time"))?;
        
        let mut next_clock_in = now.date_naive().and_hms_opt(hour, minute, 0)
            .ok_or_else(|| AppError::validation("schedule", "Invalid time"))?;
        
        // If time has passed today, schedule for tomorrow
        if next_clock_in <= now.naive_local() {
            next_clock_in = next_clock_in + chrono::Duration::days(1);
        }
        
        Ok(Local.from_local_datetime(&next_clock_in).single()
            .ok_or_else(|| AppError::validation("schedule", "Invalid time"))?
            .with_timezone(&chrono::Utc))
    }

    /// Calculate expected clock-out time
    fn calculate_expected_clock_out_time(&self, clock_in_time: &str) -> String {
        if let Ok(clock_in_dt) = DateTime::parse_from_rfc3339(clock_in_time) {
            let schedule = self.schedule.lock().unwrap();
            if let Some(schedule) = &*schedule {
                let clock_out_dt = clock_in_dt + chrono::Duration::minutes(schedule.min_work_duration_minutes as i64);
                return clock_out_dt.to_rfc3339();
            }
        }
        
        // Fallback: 9 hours from now
        (chrono::Utc::now() + chrono::Duration::hours(9)).to_rfc3339()
    }

    /// Simulate clock-in API call (replace with actual implementation)
    async fn call_clock_in_api(&self, _access_token: &str) -> Result<bool, AppError> {
        // TODO: Replace with actual API call to EMAPTA
        println!("[Scheduler] Calling clock-in API...");
        
        // Simulate network delay
        sleep(Duration::from_millis(500)).await;
        
        // Simulate success (90% success rate for testing)
        let success = rand::random::<f32>() > 0.1;
        
        if success {
            println!("[Scheduler] Clock-in API succeeded");
            Ok(true)
        } else {
            println!("[Scheduler] Clock-in API failed");
            Err(AppError::api("Clock-in API failed".to_string(), Some(500)))
        }
    }

    /// Simulate clock-out API call (replace with actual implementation)
    async fn call_clock_out_api(&self, _access_token: &str) -> Result<bool, AppError> {
        // TODO: Replace with actual API call to EMAPTA
        println!("[Scheduler] Calling clock-out API...");
        
        // Simulate network delay
        sleep(Duration::from_millis(500)).await;
        
        // Simulate success (90% success rate for testing)
        let success = rand::random::<f32>() > 0.1;
        
        if success {
            println!("[Scheduler] Clock-out API succeeded");
            Ok(true)
        } else {
            println!("[Scheduler] Clock-out API failed");
            Err(AppError::api("Clock-out API failed".to_string(), Some(500)))
        }
    }
}

// ============================================================================
// STANDALONE EXECUTION FUNCTIONS (for async tasks)
// ============================================================================

/// Execute automatic clock-in (standalone function to avoid Send issues)
async fn execute_scheduled_clock_in(
    app_handle: AppHandle,
    state: Arc<Mutex<SchedulerState>>,
    access_token: Arc<Mutex<Option<String>>>,
    schedule: Arc<Mutex<Option<WorkSchedule>>>,
    operation_id: &str,
) -> Result<(), AppError> {
    println!("[Scheduler] Executing automatic clock-in: {}", operation_id);
    
    let token = {
        let access_token = access_token.lock().unwrap();
        access_token.clone()
    };

    let result = if let Some(token) = token {
        call_clock_in_api_standalone(&token).await
    } else {
        Err(AppError::authentication("No access token available".to_string()))
    };

    let now = chrono::Utc::now().to_rfc3339();

    // Update operation status
    {
        let mut state = state.lock().unwrap();
        if let Some(operation) = state.pending_operations.iter_mut().find(|op| op.id == operation_id) {
            operation.actual_time = Some(now.clone());
            
            match result {
                Ok(true) => {
                    operation.status = "completed".to_string();
                    
                    // Update session state
                    let expected_clock_out = calculate_expected_clock_out_time_standalone(&now, &schedule);
                    state.current_session.clocked_in = true;
                    state.current_session.clock_in_time = Some(now.clone());
                    state.current_session.expected_clock_out_time = Some(expected_clock_out);
                    
                    // Emit success event
                    let _ = app_handle.emit("scheduler_event", &SchedulerEvent::ClockInSucceeded {
                        operation_id: operation_id.to_string(),
                        actual_time: now,
                    });
                }
                Ok(false) => {
                    operation.status = "failed".to_string();
                    operation.error_message = Some("Clock-in API returned false".to_string());
                    
                    let _ = app_handle.emit("scheduler_event", &SchedulerEvent::ClockInFailed {
                        operation_id: operation_id.to_string(),
                        error: "API returned false".to_string(),
                    });
                }
                Err(err) => {
                    operation.status = "failed".to_string();
                    operation.error_message = Some(err.to_string());
                    
                    let _ = app_handle.emit("scheduler_event", &SchedulerEvent::ClockInFailed {
                        operation_id: operation_id.to_string(),
                        error: err.to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Execute automatic clock-out (standalone function to avoid Send issues)
async fn execute_scheduled_clock_out(
    app_handle: AppHandle,
    state: Arc<Mutex<SchedulerState>>,
    access_token: Arc<Mutex<Option<String>>>,
    _schedule: Arc<Mutex<Option<WorkSchedule>>>,
    operation_id: &str,
) -> Result<(), AppError> {
    println!("[Scheduler] Executing automatic clock-out: {}", operation_id);
    
    let token = {
        let access_token = access_token.lock().unwrap();
        access_token.clone()
    };

    let result = if let Some(token) = token {
        call_clock_out_api_standalone(&token).await
    } else {
        Err(AppError::authentication("No access token available".to_string()))
    };

    let now = chrono::Utc::now().to_rfc3339();

    // Update operation status
    {
        let mut state = state.lock().unwrap();
        if let Some(operation) = state.pending_operations.iter_mut().find(|op| op.id == operation_id) {
            operation.actual_time = Some(now.clone());
            
            match result {
                Ok(true) => {
                    operation.status = "completed".to_string();
                    
                    // Update session state
                    state.current_session.clocked_in = false;
                    state.current_session.clock_in_time = None;
                    state.current_session.expected_clock_out_time = None;
                    
                    // Emit success event
                    let _ = app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutSucceeded {
                        operation_id: operation_id.to_string(),
                        actual_time: now,
                    });
                }
                Ok(false) => {
                    operation.status = "failed".to_string();
                    operation.error_message = Some("Clock-out API returned false".to_string());
                    
                    let _ = app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutFailed {
                        operation_id: operation_id.to_string(),
                        error: "API returned false".to_string(),
                    });
                }
                Err(err) => {
                    operation.status = "failed".to_string();
                    operation.error_message = Some(err.to_string());
                    
                    let _ = app_handle.emit("scheduler_event", &SchedulerEvent::ClockOutFailed {
                        operation_id: operation_id.to_string(),
                        error: err.to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Standalone clock-in API call
async fn call_clock_in_api_standalone(_access_token: &str) -> Result<bool, AppError> {
    // TODO: Replace with actual API call to EMAPTA
    println!("[Scheduler] Calling clock-in API...");
    
    // Simulate network delay
    sleep(Duration::from_millis(500)).await;
    
    // Simulate success (90% success rate for testing)
    let success = rand::random::<f32>() > 0.1;
    
    if success {
        println!("[Scheduler] Clock-in API succeeded");
        Ok(true)
    } else {
        println!("[Scheduler] Clock-in API failed");
        Err(AppError::api("Clock-in API failed".to_string(), Some(500)))
    }
}

/// Standalone clock-out API call
async fn call_clock_out_api_standalone(_access_token: &str) -> Result<bool, AppError> {
    // TODO: Replace with actual API call to EMAPTA
    println!("[Scheduler] Calling clock-out API...");
    
    // Simulate network delay
    sleep(Duration::from_millis(500)).await;
    
    // Simulate success (90% success rate for testing)
    let success = rand::random::<f32>() > 0.1;
    
    if success {
        println!("[Scheduler] Clock-out API succeeded");
        Ok(true)
    } else {
        println!("[Scheduler] Clock-out API failed");
        Err(AppError::api("Clock-out API failed".to_string(), Some(500)))
    }
}

/// Calculate expected clock-out time (standalone)
fn calculate_expected_clock_out_time_standalone(
    clock_in_time: &str,
    schedule: &Arc<Mutex<Option<WorkSchedule>>>,
) -> String {
    if let Ok(clock_in_dt) = DateTime::parse_from_rfc3339(clock_in_time) {
        let schedule = schedule.lock().unwrap();
        if let Some(schedule) = &*schedule {
            let clock_out_dt = clock_in_dt + chrono::Duration::minutes(schedule.min_work_duration_minutes as i64);
            return clock_out_dt.to_rfc3339();
        }
    }
    
    // Fallback: 9 hours from now
    (chrono::Utc::now() + chrono::Duration::hours(9)).to_rfc3339()
}

// Global scheduler instance
static mut SCHEDULER: Option<BackendScheduler> = None;
static SCHEDULER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global scheduler
pub fn initialize_scheduler(app_handle: AppHandle) {
    unsafe {
        SCHEDULER_INIT.call_once(|| {
            SCHEDULER = Some(BackendScheduler::new(app_handle));
        });
    }
}

/// Get the global scheduler instance
pub fn get_scheduler() -> Option<&'static BackendScheduler> {
    unsafe { SCHEDULER.as_ref() }
}

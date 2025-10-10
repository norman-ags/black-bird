use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::errors::AppError;
use crate::commands::{clock_in_api, clock_out_api, AttendanceItem};

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
}

impl BackendScheduler {

    /// Call clock-in API using shared token logic
    async fn call_clock_in_with_retry(&self) -> Result<bool, AppError> {
        crate::token_manager::clock_in_with_shared_tokens(&self.app_handle).await
    }

    /// Call clock-out API using shared token logic
    async fn call_clock_out_with_retry(&self) -> Result<bool, AppError> {
        crate::token_manager::clock_out_with_shared_tokens(&self.app_handle).await
    }

    /// Call attendance API using shared token logic
    async fn call_attendance_with_retry(&self) -> Result<Option<AttendanceItem>, AppError> {
        crate::token_manager::attendance_check_with_shared_tokens(&self.app_handle).await
    }

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
        }
    }


    /// Check and perform auto clock-in on app startup
    pub async fn check_auto_startup(&self) -> Result<bool, AppError> {
        println!("[Scheduler] Checking if auto clock-in should run...");

        // Check if we have tokens available (used for attendance API check)
        if let Err(e) = crate::token_manager::get_saved_access_token(&self.app_handle).await {
            println!("[Scheduler] No access token found, skipping auto clock-in: {}", e);
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
        
        // Check current attendance status from EMAPTA API
        println!("[Scheduler] Checking current attendance status from EMAPTA API...");
        match self.call_attendance_with_retry().await {
            Ok(Some(attendance)) => {
                println!("[Scheduler] Current attendance status: {}", attendance.attendance_status);

                // Check if it's a rest day
                if attendance.is_restday == Some(true) {
                    println!("[Scheduler] Today is a rest day, skipping auto clock-in");
                    return Ok(false);
                }

                // Check if already clocked in today (EXTERNAL CLOCK-IN HANDLING)
                if attendance.attendance_status == "Started" &&
                   attendance.date_time_in.is_some() &&
                   attendance.date_time_out.is_none() {

                    let external_clock_in = attendance.date_time_in.as_ref().unwrap();
                    println!("[Scheduler] External clock-in detected at: {}", external_clock_in);

                    // Calculate expected clock-out time
                    match self.calculate_clock_out_from_external(external_clock_in) {
                        Ok(expected_clock_out) => {
                            let now = chrono::Utc::now();

                            // Check if we're OVERDUE for clock-out
                            if now >= expected_clock_out {
                                println!("[Scheduler] OVERDUE clock-out detected! Expected: {}, Current: {}",
                                         expected_clock_out.to_rfc3339(), now.to_rfc3339());
                                println!("[Scheduler] Executing immediate clock-out...");

                                // Execute immediate clock-out (bypass minimum duration)
                                match self.manual_clock_out(true).await {
                                    Ok(success) => {
                                        if success {
                                            println!("[Scheduler] Overdue clock-out completed successfully");
                                            return Ok(true); // Report that we took action
                                        } else {
                                            println!("[Scheduler] Overdue clock-out failed");
                                            return Ok(false);
                                        }
                                    }
                                    Err(e) => {
                                        println!("[Scheduler] Error during overdue clock-out: {:?}", e);
                                        return Err(e);
                                    }
                                }
                            } else {
                                // Not overdue yet - schedule missing clock-out
                                if !self.has_pending_clock_out() {
                                    println!("[Scheduler] Scheduling missing clock-out for external clock-in");
                                    match self.schedule_clock_out_from_external(external_clock_in, expected_clock_out).await {
                                        Ok(_) => {
                                            println!("[Scheduler] Missing clock-out scheduled successfully");
                                        }
                                        Err(e) => {
                                            println!("[Scheduler] Failed to schedule missing clock-out: {:?}", e);
                                        }
                                    }
                                } else {
                                    println!("[Scheduler] Clock-out already scheduled");
                                }
                                return Ok(false); // Don't proceed with auto clock-in
                            }
                        }
                        Err(e) => {
                            println!("[Scheduler] Failed to parse external clock-in time: {:?}", e);
                            println!("[Scheduler] Skipping external clock-in handling");
                            return Ok(false);
                        }
                    }
                }

                // Check if it's a completed day
                if attendance.attendance_status == "Completed" {
                    println!("[Scheduler] Work day already completed, updating session state");

                    // Update session state to reflect completed status
                    {
                        let mut state = self.state.lock().unwrap();
                        state.current_session.clocked_in = false;
                        state.current_session.clock_in_time = attendance.date_time_in.clone();
                        state.current_session.expected_clock_out_time = attendance.date_time_out.clone();
                    }

                    return Ok(false);
                }

                // Check if on leave
                if attendance.attendance_status == "On leave" {
                    println!("[Scheduler] On leave today, skipping auto clock-in");
                    return Ok(false);
                }

                println!("[Scheduler] Attendance check passed, proceeding with auto clock-in");
            }
            Ok(None) => {
                println!("[Scheduler] No attendance record found for today, proceeding with auto clock-in");
            }
            Err(error) => {
                println!("[Scheduler] Failed to check attendance status: {}", error);
                println!("[Scheduler] Proceeding with auto clock-in anyway");
                // Don't block auto clock-in if API check fails
            }
        }
        
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

        // Call clock-in API with retry logic
        let success = self.call_clock_in_with_retry().await?;
        
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

        // Call clock-out API with retry logic
        let success = self.call_clock_out_with_retry().await?;
        
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
        let schedule_ref = Arc::clone(&self.schedule);
        let operation_id_clone = operation_id.clone();
        
        let now = chrono::Utc::now();
        let delay_seconds = clock_out_dt.timestamp() - now.timestamp();

        // Handle negative delays (past due times) and very long delays
        let delay_duration = if delay_seconds <= 0 {
            println!("[Scheduler] Clock-out time is in the past or now ({}), executing immediately", clock_out_dt.to_rfc3339());
            Duration::from_secs(1) // Execute almost immediately
        } else if delay_seconds > 86400 { // More than 24 hours
            println!("[Scheduler] WARNING: Clock-out delay is very long ({} seconds = {} hours), capping to 12 hours", delay_seconds, delay_seconds / 3600);
            Duration::from_secs(43200) // Cap at 12 hours
        } else {
            println!("[Scheduler] Clock-out scheduled in {} seconds ({:.1} hours)", delay_seconds, delay_seconds as f32 / 3600.0);
            Duration::from_secs(delay_seconds as u64)
        };
        
        let task = tokio::spawn(async move {
            sleep(delay_duration).await;
            
            // Execute clock out
            let _ = execute_scheduled_clock_out(
                app_handle,
                state,
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

    /// Calculate expected clock-out time from external clock-in (EMAPTA date format)
    fn calculate_clock_out_from_external(&self, external_clock_in: &str) -> Result<DateTime<chrono::Utc>, AppError> {
        println!("[Scheduler] Parsing external clock-in time: '{}'", external_clock_in);

        // Parse EMAPTA datetime format with improved timezone handling
        let clock_in_dt = DateTime::parse_from_rfc3339(external_clock_in)
            .or_else(|_| {
                // Try parsing without timezone info - IMPORTANT: Assume LOCAL timezone, not UTC
                if !external_clock_in.contains('T') {
                    // Format: "2024-10-09 09:00:00" -> parse as local time
                    let with_t = external_clock_in.replace(' ', "T");

                    // Try parsing as local time first (more accurate for EMAPTA times)
                    if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(&with_t, "%Y-%m-%dT%H:%M:%S") {
                        let local_dt = chrono::Local.from_local_datetime(&naive_dt).single()
                            .ok_or_else(|| AppError::validation("time", "Ambiguous local time"))?;
                        println!("[Scheduler] Parsed as local time: {} -> UTC: {}", local_dt, local_dt.with_timezone(&chrono::Utc));
                        return Ok(local_dt.with_timezone(&chrono::Utc));
                    }

                    // Fallback: treat as UTC if local parsing fails
                    let with_tz = if with_t.ends_with('Z') || with_t.contains('+') || with_t.contains('-') {
                        with_t
                    } else {
                        println!("[Scheduler] WARNING: Falling back to UTC interpretation for time: {}", external_clock_in);
                        format!("{}Z", with_t)
                    };
                    DateTime::parse_from_rfc3339(&with_tz)
                } else {
                    DateTime::parse_from_rfc3339(external_clock_in)
                }
            })
            .map_err(|e| AppError::validation("time", &format!("Invalid external clock-in time format '{}': {}", external_clock_in, e)))?;

        let schedule = self.schedule.lock().unwrap();
        let work_duration = if let Some(schedule) = &*schedule {
            schedule.min_work_duration_minutes as i64
        } else {
            540 // Default 9 hours
        };

        Ok((clock_in_dt + chrono::Duration::minutes(work_duration)).with_timezone(&chrono::Utc))
    }

    /// Schedule clock-out for external clock-in
    async fn schedule_clock_out_from_external(&self, external_clock_in: &str, expected_clock_out: DateTime<chrono::Utc>) -> Result<(), AppError> {
        println!("[Scheduler] Scheduling clock-out for external clock-in at: {}", expected_clock_out.to_rfc3339());

        let operation_id = format!("clock_out_external_{}", expected_clock_out.timestamp());

        // Update session state to reflect external clock-in
        {
            let mut state = self.state.lock().unwrap();
            state.current_session.clocked_in = true;
            state.current_session.clock_in_time = Some(external_clock_in.to_string());
            state.current_session.expected_clock_out_time = Some(expected_clock_out.to_rfc3339());

            // Add to pending operations
            state.pending_operations.push(ScheduledOperation {
                id: operation_id.clone(),
                operation_type: OperationType::ClockOut,
                scheduled_time: expected_clock_out.to_rfc3339(),
                status: "pending".to_string(),
                actual_time: None,
                error_message: None,
            });
        }

        // Create shared state for the async task
        let app_handle = self.app_handle.clone();
        let state = Arc::clone(&self.state);
        let schedule_ref = Arc::clone(&self.schedule);
        let operation_id_clone = operation_id.clone();

        let now = chrono::Utc::now();
        let delay_seconds = expected_clock_out.timestamp() - now.timestamp();

        // Handle negative delays (past due times) and very long delays
        let delay_duration = if delay_seconds <= 0 {
            println!("[Scheduler] External clock-out time is in the past or now ({}), executing immediately", expected_clock_out.to_rfc3339());
            Duration::from_secs(1) // Execute almost immediately
        } else if delay_seconds > 86400 { // More than 24 hours
            println!("[Scheduler] WARNING: External clock-out delay is very long ({} seconds = {} hours), capping to 12 hours", delay_seconds, delay_seconds / 3600);
            Duration::from_secs(43200) // Cap at 12 hours
        } else {
            println!("[Scheduler] External clock-out scheduled in {} seconds ({:.1} hours)", delay_seconds, delay_seconds as f32 / 3600.0);
            Duration::from_secs(delay_seconds as u64)
        };

        let task = tokio::spawn(async move {
            sleep(delay_duration).await;

            // Execute clock out
            let _ = execute_scheduled_clock_out(
                app_handle,
                state,
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
            scheduled_time: expected_clock_out.to_rfc3339(),
        });

        Ok(())
    }

    /// Check if we have any pending clock-out operations
    fn has_pending_clock_out(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.pending_operations.iter().any(|op| {
            matches!(op.operation_type, OperationType::ClockOut) && op.status == "pending"
        })
    }

}

// ============================================================================
// STANDALONE EXECUTION FUNCTIONS (for async tasks)
// ============================================================================


/// Call clock-in API using shared token logic (standalone)
async fn call_clock_in_with_retry_standalone(app_handle: &AppHandle) -> Result<bool, AppError> {
    crate::token_manager::clock_in_with_shared_tokens(app_handle).await
}

/// Call clock-out API using shared token logic (standalone)
async fn call_clock_out_with_retry_standalone(app_handle: &AppHandle) -> Result<bool, AppError> {
    crate::token_manager::clock_out_with_shared_tokens(app_handle).await
}

/// Execute automatic clock-in (standalone function to avoid Send issues)
async fn execute_scheduled_clock_in(
    app_handle: AppHandle,
    state: Arc<Mutex<SchedulerState>>,
    schedule: Arc<Mutex<Option<WorkSchedule>>>,
    operation_id: &str,
) -> Result<(), AppError> {
    println!("[Scheduler] Executing automatic clock-in: {}", operation_id);

    // Use storage-first pattern with retry logic
    let result = call_clock_in_with_retry_standalone(&app_handle).await;

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
    _schedule: Arc<Mutex<Option<WorkSchedule>>>,
    operation_id: &str,
) -> Result<(), AppError> {
    println!("[Scheduler] Executing automatic clock-out: {}", operation_id);

    // Use storage-first pattern with retry logic
    let result = call_clock_out_with_retry_standalone(&app_handle).await;

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

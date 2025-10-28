/**
 * Black Bird - Clock Automation
 * 
 * Main Tauri application entry point with comprehensive error handling,
 * secure storage commands, and proper application setup.
 */

// Module declarations
mod commands;
mod storage;
mod errors;
mod scheduler;
mod token_manager;
mod logging;
#[cfg(feature = "system-tray")]
mod tray;

use crate::commands::*;
use crate::errors::setup_error_handler;

/**
 * Application initialization and setup
 */
fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Setup error handling
    setup_error_handler();
    
    // Log application startup
    println!("Black Bird Clock Automation starting...");
    
    // Initialize storage backend (will be available for commands)
    let app_handle = app.handle().clone();
    match crate::storage::create_storage_backend(app_handle.clone()) {
        Ok(_) => println!("Storage backend initialized successfully"),
        Err(e) => {
            println!("ERROR: Failed to initialize storage: {}", e);
            return Err(format!("Failed to initialize storage: {}", e).into());
        }
    }
    
    // Initialize scheduler
    crate::scheduler::initialize_scheduler(app_handle.clone());
    println!("Scheduler initialized successfully");

    // Schedule automatic startup check and background monitoring initialization
    // This runs after Tauri async runtime is available
    let startup_handle = app_handle.clone();
    std::thread::spawn(move || {
        // Use std::thread to avoid Tokio runtime issues during setup
        // This will spawn a background thread that waits for Tauri to be ready
        std::thread::sleep(std::time::Duration::from_millis(3000));

        // Create a new Tokio runtime for this thread
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                println!("[Startup] Failed to create Tokio runtime for background initialization: {}", e);
                return;
            }
        };

        rt.block_on(async {
            println!("[Startup] Running automatic startup checks...");

            // Initialize background monitoring first
            match crate::commands::initialize_background_monitoring_internal(startup_handle.clone()).await {
                Ok(_) => println!("[Startup] Background monitoring initialized successfully"),
                Err(e) => println!("[Startup] WARNING: Background monitoring failed to initialize: {}", e),
            }
        });
    });

    // Initialize activity logger
    crate::logging::initialize_logger(app_handle.clone());
    println!("Activity logger initialized successfully");

    // Note: Background monitoring will be initialized automatically after Tauri starts
    // This avoids the Tokio runtime issue during synchronous setup.

    // Initialize system tray (only on supported platforms)
    #[cfg(feature = "system-tray")]
    {
        match crate::tray::create_system_tray(&app_handle) {
            Ok(_) => {
                println!("System tray initialized successfully");
            }
            Err(e) => {
                println!("WARNING: System tray failed to initialize: {}", e);
                println!("INFO: App will run normally without system tray. Window management still works.");
            }
        }
    }

    #[cfg(not(feature = "system-tray"))]
    {
        println!("INFO: System tray feature disabled (normal for development in WSL/headless environments)");
        println!("INFO: Window close behavior and all other functionality works normally.");
    }

    Ok(())
}

/**
 * Application entry point
 */
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .setup(setup_app)
        .on_window_event(|window, event| {
            use tauri::WindowEvent;
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    // Prevent window from closing, hide it instead (minimize to tray)
                    let _ = window.hide();
                    api.prevent_close();
                    println!("[Window] Close requested - minimized to tray instead");
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Storage commands
            store_encrypted_data,
            retrieve_encrypted_data,
            delete_encrypted_data,
            list_storage_keys,
            
            // Schedule commands
            set_schedule,
            get_schedule,
            
            // Scheduler commands
            start_scheduler,
            stop_scheduler,
            get_scheduler_state,
            set_scheduler_access_token,
            scheduler_manual_clock_in,
            scheduler_manual_clock_out,
            scheduler_can_clock_out,
            scheduler_check_auto_startup,
            initialize_background_monitoring,
            
            // Backend API commands
            api_exchange_refresh_token,
            api_manual_clock_in,
            api_manual_clock_out,
            api_get_attendance_status,
            api_setup_dual_tokens,

            // Autostart commands (Phase 3 Enhancement)
            enable_autostart,
            disable_autostart,
            is_autostart_enabled,

            // Activity logging commands (Phase 4 Feature)
            get_activity_logs,
            get_filtered_activity_logs,
            clear_activity_logs,
            debug_logging_status,
            reinitialize_logger,

            // Legacy greeting command (can be removed in production)
            greet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/**
 * Legacy greeting command (for compatibility)
 */
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

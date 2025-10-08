/**
 * Black Bird - Clock Automation Application
 * 
 * Main Tauri application entry point with comprehensive error handling,
 * secure storage commands, and proper application setup.
 */

// Module declarations
mod commands;
mod storage;
mod errors;

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
    match crate::storage::create_storage_backend(app_handle) {
        Ok(_) => println!("Storage backend initialized successfully"),
        Err(e) => {
            println!("ERROR: Failed to initialize storage: {}", e);
            return Err(format!("Failed to initialize storage: {}", e).into());
        }
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
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            // Storage commands
            store_encrypted_data,
            retrieve_encrypted_data,
            delete_encrypted_data,
            list_storage_keys,
            
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
    format!("Hello, {}! Welcome to Black Bird Clock Automation!", name)
}

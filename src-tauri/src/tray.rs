/**
 * System Tray Module
 *
 * Handles system tray icon, menu, and interactions for background operation.
 * Key features:
 * - Always-visible tray icon showing current status
 * - Context menu with essential actions
 * - Window hide/show management
 * - Status tooltip on hover
 */

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use crate::scheduler::get_scheduler;

/// Create the system tray with context menu
pub fn create_system_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // Create context menu for the tray
    let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Black Bird", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&show_hide, &separator, &quit])?;

    // Attempt to create tray icon - this may fail on systems without proper GUI support
    println!("[Tray] Attempting to create system tray icon...");
    match TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Black Bird - Clock Automation")
        .menu(&menu)
        .show_menu_on_left_click(false) // Menu on right-click only
        .on_tray_icon_event(|tray, event| {
            handle_tray_event(tray.app_handle(), event)
        })
        .on_menu_event(|app, event| {
            handle_menu_event(app, event)
        })
        .build(app) {
        Ok(_tray) => {
            println!("[Tray] System tray created successfully with context menu");
            Ok(())
        }
        Err(e) => {
            println!("[Tray] Failed to create system tray: {}", e);
            Err(e)
        }
    }
}

/// Handle tray icon events
fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            // Left click: Show/hide main window
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                    println!("[Tray] Window hidden via tray click");
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                    println!("[Tray] Window shown via tray click");
                }
            }
        }
        TrayIconEvent::Click {
            button: MouseButton::Right,
            button_state: MouseButtonState::Up,
            ..
        } => {
            // Right-click will automatically show the context menu (handled by Tauri)
            println!("[Tray] Right-click detected - context menu should appear");
        }
        _ => {}
    }
}

/// Handle tray menu events
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "show_hide" => {
            // Toggle window visibility
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                    println!("[Tray] Window hidden via menu");
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                    println!("[Tray] Window shown via menu");
                }
            }
        }
        "quit" => {
            // Quit the application
            println!("[Tray] Quit requested via menu");
            app.exit(0);
        }
        _ => {}
    }
}

/// Update tray status based on current scheduler state
pub fn update_tray_status<R: Runtime>(app: &AppHandle<R>, status_text: &str) {
    if let Some(tray) = app.tray_by_id("main") {
        let new_tooltip = format!("Black Bird - {}", status_text);
        let _ = tray.set_tooltip(Some(&new_tooltip));

        // Update status menu item if possible
        // Note: Updating menu items in real-time requires rebuilding the menu in Tauri v2
        // For now, we'll focus on tooltip updates

        println!("[Tray] Status updated: {}", status_text);
    }
}

/// Get current status text from scheduler
pub fn get_current_status_text() -> String {
    if let Some(scheduler) = get_scheduler() {
        let state = scheduler.get_state();

        if state.current_session.clocked_in {
            if let Some(clock_in_time) = &state.current_session.clock_in_time {
                // Parse time and show simple format
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(clock_in_time) {
                    let local_time = dt.with_timezone(&chrono::Local);
                    return format!("Clocked in at {}", local_time.format("%I:%M %p"));
                }
            }
            "Currently working".to_string()
        } else {
            if state.is_running {
                "Ready for work".to_string()
            } else {
                "Not running".to_string()
            }
        }
    } else {
        "Initializing...".to_string()
    }
}
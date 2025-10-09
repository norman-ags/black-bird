# Plan: EMAPTA Clock Automation Desktop App

## Overview

Build a simple, user-friendly internal desktop application using Tauri to automatically handle daily clock in/out through the EMAPTA API.

## Design Philosophy

**Set it and forget it**: User provides refresh token once, then the app handles everything automatically.

## Simplified User Flow

### 1. One-Time Setup (First Launch Only)

- **First time**: User enters refresh token on initial launch
- **Subsequent launches**: App checks for stored token and skips setup
- App exchanges refresh token for access token and saves both securely
- Setup complete - no further user configuration needed

### 2. Daily Automatic Operation

- **App startup**: Check if stored token exists, skip setup if found
- **Token validation**: Refresh access token if needed, save new tokens
- **Attendance check**: Verify if can clock in today (not same day, not rest day, not on leave)
- **Auto clock-in**: If eligible, automatically clock in with token refresh and save
- **Background operation**: Continue running when window closed (system tray)
- **Auto clock-out**: After exactly 9 hours, clock out with token refresh and save
- **Sleep/wake handling**: Gap detection triggers auto clock-in on new days
- **Next day**: Repeat cycle when app detects new day (via gap detection or restart)

### 3. Token Management (Automatic)

- **Storage**: Both `refresh_token` and `access_token` stored securely
- **Auto-refresh**: Tokens refreshed during clock operations (in/out)
- **Persistence**: New tokens saved immediately after refresh
- **Background updates**: Token refresh happens in backend, no UI interruption

### 4. User Experience Flow

1. **User opens app** → Check stored token → Skip setup if found → Go to status screen
2. **User enters refresh token** → Exchange for access token → Save both → Auto clock-in
3. **App automatically clocks in** → Background operation → Token refresh during operation
4. **User closes app** → Minimizes to system tray → Continues background operation
5. **App automatically clocks out** → After 9 hours → Token refresh → Save new tokens
6. **User sleeps computer** → App continues in tray → Gap detection monitoring
7. **Next day wake up** → Gap detection triggers → Auto clock-in → Token refresh/save
8. **User opens from tray** → Status screen (setup already complete)

### 5. Manual Override (Emergency Use)

- Manual clock in/out buttons available if needed
- For exceptional circumstances only

## Technical Flow

### Authentication & Token Management

#### Initial Setup (One-Time)

```
POST https://api.platform.emapta.com/auth/v1/auth/protocol/openid-connect/token
Headers:
  content-type: application/json
Body:
  {
    "grant_type": "refresh_token",
    "client_id": "EMAPTA-MYEMAPTAWEB",
    "refresh_token": "<user_provided_refresh_token>",
    "scope": "openid"
  }
```

#### Shared Token Logic (Updated Approach)

- **Primary Pattern**: Always use saved `access_token` for API calls first
- **Retry Logic**: Only refresh tokens on token-related errors (401, invalid_token)
- **Single Retry**: Refresh → Save → Retry once → If fails, do nothing
- **Storage Strategy**: Fixed keys overwrite previous tokens (no token versioning)
- **Efficiency**: Minimize token refresh calls (only when actually needed)

#### Token Storage Strategy

- **Location**: Secure encrypted storage (Tauri's secure storage)
- **Keys**: `refresh_token`, `access_token`
- **Auto-update**: Both tokens updated after each API operation
- **Persistence**: Tokens survive app restarts, sleep/wake cycles

#### App Launch Logic

1. **Check storage**: Look for existing `refresh_token`
2. **If found**: Skip setup → Validate/refresh → Go to status screen
3. **If not found**: Show setup screen → Get refresh token → Exchange → Save → Status screen

### Automatic Attendance Tracking Flow

#### 1. **Launch**
- User opens the app

#### 2. **Authentication**
- User enters refresh token and access token securely
- Save both tokens in **fixed storage keys** (overwrite originals)

#### 3. **Attendance Check**
- Use saved **access token** to call Attendance API
- **If API call fails (token-related error):**
  1. Refresh tokens (new refresh + new access token)
  2. Overwrite existing storage keys with new tokens
  3. Retry Attendance API call **once** with new access token
- If retry succeeds: Continue logic to decide if clock-in needed

#### 4. **Clock-In**
- If clock-in required:
  - Use **current saved access token** for clock-in API
  - Success should follow (token already validated during attendance check)
  - If clock-in unexpectedly fails: repeat refresh & save (single retry)

#### 5. **Minimize / Close to Tray**
- Closing window minimizes to system tray
- App continues running in background

#### 6. **Automatic Clock-Out**
- After 9 hours since last successful clock-in:
  - Use **current saved access token** for clock-out
  - If token error → refresh, overwrite keys, retry once

#### 7. **Sleep / Wake Handling**
- On waking: run Attendance API check with saved access token
- Apply refresh/overwrite/retry logic if token error occurs

### Auto Clock In

```
POST https://api.platform.emapta.com/time-and-attendance/ta/v1/dtr/attendance/login
Headers:
  Authorization: Bearer <access_token>
  content-type: application/json
```

- Executed automatically on app startup if eligible
- Show notification of successful clock-in

### Auto Clock Out (After 9 Hours)

```
POST https://api.platform.emapta.com/time-and-attendance/ta/v1/dtr/attendance/logout
Headers:
  Authorization: Bearer <access_token>
  content-type: application/json
```

- Executed automatically exactly 9 hours after clock-in
- Show notification of successful clock-out

## Background Operation & System Tray

### Core Requirement: Always Running

- **Background operation**: App continues running when window is closed
- **System tray**: App minimizes to system tray (Windows notification area)
- **Tray interactions**:
  - Left click: Show/hide main window
  - Right click: Context menu (Status, Open, Settings, Exit)
- **Auto-start**: Launch with system startup (optional setting)

### System Tray Features:

- **Status icon**: Visual indicator of current state (clocked in/out)
- **Tooltip**: Quick status on hover ("Clocked in at 9:15 AM")
- **Notifications**: Native system notifications for all clock events
- **Always accessible**: User can access app even when main window closed

### System Tray Implementation Requirements

#### **Dependencies**
```toml
# src-tauri/Cargo.toml
tauri = { version = "2", features = ["system-tray"] }
```

#### **Tray Menu Structure**
- **📊 Status** - Show current clock state
- **📱 Show Window** - Restore main window
- **⚙️ Settings** - Quick settings access
- **❌ Exit** - True application exit

#### **Window Management Behavior**
- **Close Button (X)**: Minimizes to tray (prevents exit)
- **Minimize Button**: Also minimizes to tray
- **Restore**: Single click tray icon OR click "Show Window"
- **Context Menu**: Right-click tray icon
- **Exit**: Only via tray context menu "Exit" option

#### **Background Operation Requirements**
- **Scheduler Persistence**: All automatic operations continue when window hidden
- **Token Management**: Shared token logic works identically in background
- **API Operations**: Clock-in/out, attendance checking continues seamlessly
- **Gap Detection**: Sleep/wake monitoring maintains functionality
- **Event Notifications**: System notifications for all clock events

## User Interface (Ultra-Minimal)

### Single Screen Interface:

1. **🔑 Setup** (only if not configured)
   - Refresh token input
   - Save button
2. **📊 Status** (only screen after setup)
   - Current status: "Not clocked in" / "Clocked in at X" / "Shift Completed"
   - Time until auto clock-out (if clocked in)
   - Today's work summary
   - **⚠️ Manual Override** (emergency only): Manual clock in/out buttons

### Removed Features:

- ~~Advanced mode toggle~~ (eliminated completely)
- ~~Schedule configuration UI~~ (backend handles automatically)
- ~~Multiple tabs~~ (single status screen only)
- ~~Manual clock controls~~ (emergency override only)

### Window Behavior:

- **Minimize to tray**: Clicking X minimizes instead of closing
- **Restore from tray**: Click tray icon to restore window
- **True exit**: Only via tray context menu "Exit" option

### Notifications

- System tray notifications for clock in/out
- Error notifications if API calls fail
- Success confirmations
- Click notification to open main window

## Removed Features (Simplified)

- ~~Auto schedule configuration~~ (keep code but hide from main UI)
- ~~Complex timezone settings~~ (use system timezone)
- ~~Detailed logging interface~~ (basic logging in background)
- ~~Manual schedule adjustments~~ (9 hours fixed)

## Core Logic

### Backend Auto Clock-In System:

- **Backend responsibility**: All auto clock-in logic runs in Tauri backend
- **Persistent operation**: Works even when frontend is closed/restarted
- **Default schedule**: Fixed 9-hour work day, auto-detects appropriate clock-in time
- **Smart detection**: Checks rest days, leave status, and previous clock-ins

### App Startup Sequence:

1. **Backend initialization**: Scheduler starts and checks current state
2. **Token check**: Look for stored `refresh_token` in secure storage
3. **Frontend routing**: If token found → Status screen, If not found → Setup screen
4. **Token validation**: Refresh access token if needed, save new tokens
5. **Attendance check**: Backend checks if today is a work day
6. **Auto clock-in**: Backend automatically clocks in if eligible (with token refresh/save)
7. **Gap detection**: Start background monitoring for sleep/wake scenarios

### Backend Operations (Always Running):

- **Auto clock-in**: Check and execute on app startup and daily schedule
- **Auto clock-out**: Fixed 9-hour timer from clock-in time
- **Shared token logic**: Use saved access token first, refresh only on errors
- **Attendance API checks**: Rest day/leave detection with retry pattern
- **State persistence**: Maintains state and tokens across app restarts
- **Event notifications**: Send status updates to frontend/tray
- **Gap detection**: Monitor for sleep/wake cycles to trigger daily auto clock-in

### Token Management Flow Corrections

#### **Current Problem:**
```rust
// ❌ Current: Always refresh before API calls (inefficient)
get_fresh_access_token() → exchange_refresh_token_api() → api_call()
```

#### **Required Fix:**
```rust
// ✅ Fixed: Try saved token first, refresh only on errors
storage.retrieve("access_token") → api_call()
  → if token_error: refresh_and_save() → retry_once()
```

#### **Universal API Pattern:**
```rust
async fn api_with_shared_tokens<T, F>(
    app_handle: &AppHandle,
    operation: F,
    operation_name: &str,
) -> Result<T, AppError>
where F: Fn(&str) -> Future<Result<T, String>>
{
    let storage = create_storage_backend(app_handle.clone())?;

    // 1. Try with saved access token
    let access_token = storage.retrieve("access_token").await?
        .ok_or("No access token found")?;

    match operation(&access_token).await {
        Ok(result) => Ok(result),
        Err(error) if is_token_error(&error) => {
            // 2. Token error: refresh and overwrite keys
            let new_tokens = refresh_and_save_tokens(app_handle).await?;

            // 3. Retry once with new token
            operation(&new_tokens.access_token).await
                .map_err(|e| AppError::api(format!("{} retry failed: {}", operation_name, e), None))
        }
        Err(error) => Err(AppError::api(error, None))
    }
}
```

#### **Token Error Detection:**
```rust
fn is_token_error(error: &str) -> bool {
    error.contains("401") ||
    error.contains("unauthorized") ||
    error.contains("invalid_token") ||
    error.contains("token_expired")
}
```

#### **Fixed Storage Keys (Never Change):**
```rust
const REFRESH_TOKEN_KEY: &str = "refresh_token";
const ACCESS_TOKEN_KEY: &str = "access_token";
```

### Frontend Responsibilities (Minimal):

- **Status display**: Show current clock state and countdown
- **Emergency controls**: Manual override buttons for exceptional cases
- **Setup interface**: Token input on first run
- **Tray integration**: System tray presence and notifications

### Window Management:

- **Close button (X)**: Minimizes to tray instead of exiting
- **True exit**: Only via tray context menu → "Exit"
- **Restore**: Click tray icon or notification to show window
- **Auto-minimize**: Option to start minimized to tray

## Implementation Strategy

### Current State - COMPLETED ✅

- ✅ **Phase 1**: Shared Token Logic Foundation (Universal token manager, retry patterns)
- ✅ **Phase 2**: System Tray Integration (Minimize to tray, context menu, background operation)
- ✅ **Phase 3**: Authentication Flow Enhancement (Dual token setup, auto-launch integration)
- ✅ **Phase 4**: Production Polish (External clock-in detection, overdue protection)

### **Windows Deployment Fixes (October 2025)**

#### **Issues Discovered During Windows Testing:**
1. **Token Setup Screen Persistence**: After saving tokens, app remained on setup screen
2. **System Tray Right-Click Missing**: No context menu on right-click
3. **Cross-Platform Build**: Need Windows binaries with system tray support

#### **Fixes Applied:**
1. **Enhanced AuthProvider** (`src/provider/AuthProvider.tsx`):
   - Added `reloadTokens()` function to refresh token state from storage
   - TokenSetup now calls `reloadTokens()` after successful save
   - Proper state synchronization between storage and UI

2. **System Tray Context Menu** (`src-tauri/src/tray.rs`):
   - Added full context menu with "Show/Hide Window" and "Quit" options
   - Right-click functionality with proper menu event handling
   - `app.exit(0)` for true application termination

3. **Windows Cross-Compilation** (WSL → Windows):
   - MinGW-w64 toolchain configured for Windows builds
   - Cross-compilation produces: `black-bird.exe` + `WebView2Loader.dll`
   - System tray feature enabled via `--features system-tray` flag

#### **Windows Build Instructions:**
```bash
# Cross-compile from WSL/Linux to Windows
npm run tauri build -- --features system-tray --target x86_64-pc-windows-gnu

# Output location:
# target/x86_64-pc-windows-gnu/release/black-bird.exe
# target/x86_64-pc-windows-gnu/release/WebView2Loader.dll
```

## Implementation Phases for Scenario Completion

### **Phase 1: Shared Token Logic Foundation** (Week 1)

#### **Priority: CRITICAL** - Required for scenario compliance

1. **Create Universal Token Manager** (`src-tauri/src/token_manager.rs`)
   - `get_saved_access_token()` - Get current token from storage
   - `refresh_and_save_tokens()` - Refresh and overwrite storage keys
   - `api_with_shared_tokens<T, F>()` - Universal API call pattern
   - `is_token_error()` - Detect 401/invalid_token errors

2. **Update All API Operations**
   - Replace current eager refresh pattern with "try saved → refresh on error → retry once"
   - Apply to: attendance check, clock-in, clock-out, manual commands
   - Remove duplicate token refresh implementations

3. **Fix Scheduler Token Usage**
   - Remove `get_fresh_access_token()` (always refreshes)
   - Use shared token helper for all scheduler operations
   - Eliminate in-memory token management completely

### **Phase 2: System Tray Integration** (Week 2)

#### **Priority: HIGH** - Required for "minimize to tray" scenario step

1. **Add System Tray Dependencies**
   - Update `src-tauri/Cargo.toml` with tray features
   - Create `src-tauri/src/tray.rs` module

2. **Implement Window Management Override**
   - Intercept close button to minimize instead of exit
   - Handle tray icon click to restore window
   - Create tray context menu (Status, Show, Exit)

3. **Background Operation Verification**
   - Test all automatic operations continue when window hidden
   - Verify scheduler maintains 9-hour timer in background
   - Ensure gap detection works when minimized

### **Phase 3: Authentication Flow Enhancement** (Week 3)

#### **Priority: MEDIUM** - Polish for production readiness

1. **Token Setup Improvement**
   - Support both refresh + access token input during setup
   - Validate tokens on initial setup
   - Clear error messages for invalid tokens

2. **Auto-Launch Integration**
   - System startup registration (Windows/macOS/Linux)
   - Launch minimized option
   - Startup behavior configuration

### **Phase 4: Production Polish** (Week 4)

#### **Priority: LOW** - Final touches

1. **Native Notifications**
   - System notifications for clock events
   - Notification click to restore window
   - Error notifications for failures

2. **Enhanced Error Recovery**
   - Network connectivity handling
   - API timeout and retry logic
   - User-friendly error messages

3. **Final UI Streamlining**
   - Remove any remaining advanced features
   - Emergency manual override only
   - Ultra-minimal status display

### **Success Criteria for Scenario Completion**

✅ **Shared Token Logic**: All operations use saved access token first, refresh only on errors
✅ **System Tray**: Close button minimizes to tray, app continues in background
✅ **Automatic Operations**: 9-hour clock-out works in background
✅ **Sleep/Wake**: Gap detection triggers attendance check with shared tokens
✅ **Single Retry**: All API failures retry exactly once after token refresh
✅ **Fixed Storage**: All new tokens overwrite existing keys consistently

## Target User Experience

### First Launch:

1. App opens to setup screen
2. User enters refresh token
3. App tests connection and saves tokens
4. App shows "Setup complete! App will run in background and handle everything automatically."
5. App minimizes to system tray

### Daily Use:

1. User starts computer (app auto-launches in tray)
2. App automatically checks and clocks in if eligible
3. Tray notification: "Clocked in at 9:15 AM. Will clock out at 6:15 PM."
4. App runs silently in background
5. At 6:15 PM: Auto clock-out + tray notification
6. User shuts down computer, cycle repeats tomorrow

### Accessing the App:

1. **View status**: Hover over tray icon for quick status
2. **Open window**: Click tray icon to show main window
3. **Quick actions**: Right-click tray for context menu

### Emergency Override:

1. Click tray icon to open main window
2. Click "Manual Override" button
3. Manual clock in/out options available
4. Returns to automatic mode

## Success Metrics

- Zero daily user interaction required after setup
- Reliable 9-hour work day tracking
- Clear status visibility when needed
- Foolproof operation for non-technical users

## Technical Specifications for Background Operation

### **System Tray Architecture**

#### **Tauri Configuration**
```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .system_tray(create_system_tray())
        .on_system_tray_event(handle_tray_event)
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                // Prevent close, minimize to tray instead
                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
}
```

#### **Background Scheduler Requirements**
- **Persistent Timers**: 9-hour clock-out timer survives window hide/show
- **API Operations**: All attendance/clock operations continue seamlessly
- **Token Management**: Shared token helper works identically in background
- **Gap Detection**: Sleep/wake monitoring maintains 5-minute check interval
- **State Persistence**: Current session state preserved during window operations

### **Token Storage Architecture**

#### **Fixed Key Strategy**
```rust
// Never change these keys - always overwrite existing values
const REFRESH_TOKEN_KEY: &str = "refresh_token";
const ACCESS_TOKEN_KEY: &str = "access_token";

// All operations use these same keys for consistency
pub async fn save_tokens(storage: &Storage, tokens: &TokenResponse) {
    storage.store(REFRESH_TOKEN_KEY, &tokens.refresh_token).await?;
    storage.store(ACCESS_TOKEN_KEY, &tokens.access_token).await?;
}
```

#### **API Integration Pattern**
```rust
// Applied to ALL API operations:
// 1. attendance_check_with_shared_tokens()
// 2. clock_in_with_shared_tokens()
// 3. clock_out_with_shared_tokens()
// 4. manual_clock_operations()

async fn attendance_check_with_shared_tokens(app_handle: &AppHandle) -> Result<Option<AttendanceItem>, AppError> {
    api_with_shared_tokens(app_handle, |token| get_attendance_status_api(token), "attendance_check").await
}
```

### **Background Operation Validation**

#### **Test Scenarios**
1. **Window Hidden**: Verify 9-hour timer continues, API calls succeed
2. **System Sleep**: Ensure gap detection triggers attendance check on wake
3. **Network Reconnect**: Token refresh works after connectivity issues
4. **Tray Restore**: Window state and scheduler status consistent after restore

#### **Performance Requirements**
- **Token Efficiency**: Max 1 refresh per API operation (not per call)
- **Memory Usage**: Scheduler maintains minimal state when window hidden
- **CPU Usage**: Gap detection uses 5-minute intervals (not continuous polling)
- **Storage Access**: Batch token saves to minimize disk I/O

### **Error Handling in Background**

#### **Silent Operation Requirements**
- **API Failures**: Log errors, continue operation, no user interruption
- **Token Refresh Failures**: Single retry, then skip operation silently
- **Network Issues**: Wait for next scheduled operation, don't accumulate retries
- **Storage Errors**: Fallback to in-memory operation, attempt save on next success

This ensures your automatic attendance tracking scenario works seamlessly with true background operation and shared token management.

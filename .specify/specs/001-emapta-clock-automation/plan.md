# Plan: EMAPTA Clock Automation Desktop App

## Overview

Build a simple, user-friendly internal desktop application using Tauri to automatically handle daily clock in/out through the EMAPTA API.

## Design Philosophy

**Set it and forget it**: User provides refresh token once, then the app handles everything automatically.

## Simplified User Flow

### 1. One-Time Setup

- User enters refresh token on first launch
- App exchanges for access token and saves both securely
- Setup complete - no further configuration needed

### 2. Daily Automatic Operation

- **App startup**: Check if can clock in today (not same day, not rest day, not on leave)
- **Auto clock-in**: If eligible, automatically clock in immediately
- **Wait 9 hours**: Track time automatically in background
- **Auto clock-out**: Automatically clock out after exactly 9 hours
- **Next day**: Repeat the cycle when app opens

### 3. Manual Override (Emergency Use)

- Manual clock in/out buttons available if needed
- For exceptional circumstances only

## Technical Flow

### Authentication (One-Time)

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

- Save both `access_token` and new `refresh_token` securely
- Auto-refresh tokens when needed

### Daily Attendance Check

```
GET https://api.platform.emapta.com/time-and-attendance/ta/v1/dtr/attendance?date_from=<today>&date_to=<today>
Headers:
  Authorization: Bearer <access_token>
  content-type: application/json
```

- Check if today is a work day (not "on leave" or "rest day")
- Check if not already clocked in today
- Only proceed if eligible

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

## User Interface (Ultra-Minimal)

### Single Screen Interface:

1. **🔑 Setup** (only if not configured)
   - Refresh token input
   - Save button
2. **📊 Status** (only screen after setup)
   - Current status: "Not clocked in" / "Clocked in at X" / "Clocked out at Y"
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
2. **Token validation**: Refresh access token if needed
3. **Attendance check**: Backend checks if today is a work day
4. **Auto clock-in**: Backend automatically clocks in if eligible
5. **Frontend display**: Shows current status (clocked in/out)

### Backend Operations (Always Running):

- **Auto clock-in**: Check and execute on app startup and daily schedule
- **Auto clock-out**: Fixed 9-hour timer from clock-in time
- **Token refresh**: Automatic when expired
- **Attendance API checks**: Rest day/leave detection
- **State persistence**: Maintains state across app restarts
- **Event notifications**: Send status updates to frontend/tray

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

### Current State

- ✅ Token management and storage (US1)
- ✅ Manual clock operations (US1)
- ✅ Backend scheduler architecture (US2)
- ✅ Schedule configuration UI (US2) - **Keep but simplify**

### Required Changes for Backend Migration

#### High Priority Changes:

1. **Remove Advanced Mode**: Eliminate advanced mode toggle from App.tsx completely
2. **Backend Auto-startup**: Move auto clock-in logic from frontend useAutoStartup hook to backend scheduler initialization
3. **Simplified frontend**: Update StatusScreen to be pure display component
4. **Backend integration**: Update backend scheduler to handle auto clock-in on initialization
5. **State management**: Remove frontend auto-startup state management

#### Medium Priority (After Core Simplification):

1. **System tray integration**: Always-running background app with tray presence
2. **Window management**: Close-to-tray behavior instead of exit
3. **Startup behavior**: Launch on system startup (optional)
4. **Error recovery**: Robust handling of network/API issues
5. **Tray context menu**: Status, settings, manual override, exit options

#### Eliminated Features:

1. **Advanced mode toggle**: Removed completely from UI
2. **Schedule configuration UI**: Backend handles automatically
3. **Frontend auto-startup logic**: Moved to backend
4. **Complex manual controls**: Emergency override only

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

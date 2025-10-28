# Development Notes

**Session Date:** 2025-10-23
**Focus:** Windows Clock-in Issues & Debugging Infrastructure

## 📝 Current Session Work
<!-- Update this section each time you start working -->

**Working On:**
- Fixed manual clock-in errors on Windows
- Fixed automatic clock-in after laptop wake from sleep
- Added comprehensive debugging infrastructure

**Goals:**
- [x] Fix backend race conditions in scheduler initialization
- [x] Enhance error handling and logging throughout the app
- [x] Add debug logs UI for better visibility into failures
- [x] Improve wake detection robustness

## 🚧 Active Issues/Blockers
<!-- Add any problems you encounter -->

*None currently - add issues as they arise*

## ✅ Completed Today
<!-- Track daily progress -->

- **Fixed Backend Race Conditions**: Added scheduler initialization waiting with retries (src-tauri/src/commands.rs:407-432)
- **Enhanced Token Validation**: Improved wake detection with robust token checking before operations
- **Added Debug Logs UI**: Created comprehensive logs viewer (src/components/DebugLogs.tsx) with filtering and real-time updates
- **Improved Error Handling**: Added proper error logging throughout the application
- **Frontend Error Reporting**: Enhanced console logging for manual operations (src/hooks/use-clock.ts)
- **Fixed Compilation Issues**: Resolved log_error method calls to use proper logging interface

## 🔍 Key Insights/Decisions
<!-- Document important architectural decisions or discoveries -->

- **Storage-First Pattern**: All operations should follow `storage.retrieve("refresh_token") → exchange → storage.store(new_tokens) → API_call`
- **Consistent Retry Logic**: "Attempt → if fails → Refresh → Retry once → if still fails → Return error"
- **Backend Reliability**: Scheduler operations must survive frontend reloads
- **Windows Debugging**: Race conditions in scheduler initialization were causing wake-detection failures
- **Logging Architecture**: Built-in logging system (src-tauri/src/logging.rs) provides structured logs with monthly rotation
- **Error Flow Discovery**: Manual vs Auto clock-in use different but converging code paths, both ending at token_manager::clock_in_with_shared_tokens

## 🎯 Next Steps
<!-- What to work on next -->

1. **Test on Windows**: Verify the fixes work in Windows environment
2. **Monitor wake detection**: Test laptop sleep/wake behavior with new logging
3. **Token validation**: Ensure token refresh works correctly in all scenarios
4. **UI refinements**: Improve debug logs interface based on user feedback
5. **Performance**: Monitor if frequent logging impacts app performance

## 📚 Useful Code Locations
<!-- Reference key files and line numbers -->

- **Token Management Pattern**: `src-tauri/src/commands.rs` lines 380-383 (api_manual_clock_in)
- **Background Monitoring**: `src-tauri/src/commands.rs` lines 404-546 (initialize_background_monitoring_impl)
- **Scheduler Structure**: `src-tauri/src/scheduler.rs` lines 100-445 (manual_clock_in/out methods)
- **Logging System**: `src-tauri/src/logging.rs` (ActivityLogger implementation)
- **Debug UI**: `src/components/DebugLogs.tsx` (logs viewing interface)
- **Frontend Clock Logic**: `src/hooks/use-clock.ts` lines 7-37

## 🐛 Known Bugs
<!-- Track bugs that need fixing -->

- **Windows tray functionality disabled** (needs libappindicator, Linux-specific)
- **Rust warnings** about unused imports and static references (non-critical)
- **Token validation might still fail** if tokens are invalid/expired (now has better error reporting)

## 💡 Future Improvements
<!-- Ideas for later implementation -->

- System tray integration for true background operation
- Native notifications for clock events
- Auto-launch with system startup
- Enhanced error recovery and logging

---

**Usage Instructions:**
- Update this file at start/end of each development session
- Keep "Current Session Work" updated with what you're actively doing
- Document decisions and insights for future reference
- Track progress and blockers for better continuity
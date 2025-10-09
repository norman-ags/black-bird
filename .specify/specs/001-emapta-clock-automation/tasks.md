# Tasks: EMAPTA Clock Automation

**Input**: Design documents from `.specify/specs/001-emapta-clock-automation/`
**Prerequisites**: plan.md (avai- [x] T107 [Backend] Remove advanced mode completely: eliminate advanced mode toggle from App.tsx and simplify to single status screen with emergency manual override only. ✅

- [x] T108 [Backend] Move auto clock-in to backend: migrate auto-startup logic from frontend useAutoStartup hook to backend scheduler initialization for better reliability. ✅
- [x] T109 [Backend] Update StatusScreen for pure display: remove auto-startup integration from frontend component, make it display-only with emergency manual controls. ✅le), spec.md (available)

**Tests**: Tests are NOT explicitly requested in the feature specification, so test tasks are omitted per instructions.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md, this is a single Tauri desktop application with paths at repository root:

- **Source Code**: `src/` for React frontend, `src-tauri/` for Rust backend
- **Tests**: `tests/` directory
- **Docs**: `docs/` directory

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic Tauri + React + TypeScript structure

- [x] T001 Initialize Tauri project with React and TypeScript template
- [x] T002 [P] Configure package.json with required dependencies (React 18+, TypeScript 5.x, date-fns)
- [x] T003 [P] Setup Biome configuration for linting and formatting in biome.json
- [x] T004 [P] Configure TypeScript strict mode in tsconfig.json
- [x] T005 [P] Setup Tauri configuration in src-tauri/tauri.conf.json with app permissions
- [x] T006 [P] Create project directory structure per plan.md (src/components/, src/services/, etc.)
- [x] T007 [P] Setup Vite configuration for React development
- [x] T008 [P] Configure Tauri build settings and app icons in public/icons/

**Checkpoint**: Basic project structure ready for development

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T009 Create base TypeScript interfaces in src/types/auth.ts for token management
- [x] T010 [P] Create base TypeScript interfaces in src/types/api.ts for EMAPTA API responses
- [x] T011 [P] Create base TypeScript interfaces in src/types/clock.ts for clock operations
- [x] T012 [P] Create base TypeScript interfaces in src/types/schedule.ts for scheduling
- [x] T013 Create EMAPTA API endpoints constants in src/constants/api-endpoints.ts
- [x] T014 Implement secure storage service using Tauri filesystem APIs in src/services/storage-service.ts
- [x] T015 Create crypto utilities for token encryption/decryption in src/utils/crypto.ts
- [x] T016 [P] Create validation utilities in src/utils/validation.ts for parameter checking
- [x] T017 [P] Create timezone utilities using date-fns in src/utils/timezone.ts
- [x] T018 Implement base API client with retry logic in src/services/api-client.ts
- [x] T019 Setup Tauri commands for secure storage operations in src-tauri/src/commands.rs
- [x] T020 Implement Tauri storage backend in src-tauri/src/storage.rs
- [x] T021 Create main Tauri application setup in src-tauri/src/main.rs
- [x] T022 Setup error handling framework and logging infrastructure
- [x] T023 Create main App.tsx component structure with routing

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic Token Setup and Manual Clock Operations (Priority: P1) 🎯 MVP

**Goal**: User can provide refresh token and manually clock in/out to verify API connection works

**Independent Test**: Enter valid refresh token, click clock in/out buttons, see successful API responses

### Implementation for User Story 1

- [x] T024 [P] [US1] Create TokenSetup component in src/components/TokenSetup.tsx for token input and validation
- [x] T025 [P] [US1] Create ClockControls component in src/components/ClockControls.tsx for manual clock operations
- [x] T026 [US1] Implement authentication service in src/services/auth-service.ts with token exchange logic
- [x] T027 [US1] Implement clock service in src/services/clock-service.ts for manual clock in/out operations
- [x] T028 [US1] Create useAuth custom hook in src/hooks/use-auth.ts for token state management
- [x] T029 [US1] Create useClock custom hook in src/hooks/use-clock.ts for clock operation state
- [x] T030 [US1] Integrate TokenSetup component with authentication service for token validation
- [x] T031 [US1] Integrate ClockControls component with clock service for manual operations
- [x] T032 [US1] Add error handling and user feedback for invalid tokens and API failures
- [x] T033 [US1] Add JSDoc documentation to all US1 functions and components
- [x] T034 [US1] Update App.tsx to integrate TokenSetup and ClockControls components

**Checkpoint**: At this point, User Story 1 should be fully functional - users can store tokens and manually clock in/out

---

## Phase 4: User Story 2 - Automatic Scheduling with 9-Hour Minimum (Priority: P2)

**Goal**: User can set preferred clock-in time and system automatically handles clock-in/out with 9-hour minimum

**Independent Test**: Set clock-in time to 9:00 AM, verify automatic clock-in at 9:00 AM and clock-out at 6:00 PM

### Implementation for User Story 2

- [x] T035 [P] [US2] Create ScheduleConfig component in src/components/ScheduleConfig.tsx for time configuration ✅
- [x] T036 [US2] Implement schedule service in src/services/schedule-service.ts with 9-hour minimum logic ✅
- [x] T037 [US2] Create useSchedule custom hook in src/hooks/use-schedule.ts for schedule state management ✅
<!-- - [ ] T038 [US2] Add background task scheduling using Tauri's system tray or intervals
- [ ] T039 [US2] Implement automatic clock-in functionality with timezone handling
- [ ] T040 [US2] Implement automatic clock-out functionality with 9-hour minimum enforcement
- [ ] T041 [US2] Add retry logic for failed automatic operations with exponential backoff
- [ ] T042 [US2] Add manual override capabilities for automatic scheduling
- [ ] T043 [US2] Create notification system for successful/failed automatic operations
- [ ] T044 [US2] Integrate ScheduleConfig component with schedule service
- [ ] T045 [US2] Add JSDoc documentation to all US2 functions and components
- [ ] T046 [US2] Update App.tsx to integrate ScheduleConfig component -->

### Recommendation & incremental tasks

Recommendation: keep the ScheduleConfig UI and schedule editing in the renderer (React), but add a small set of backend (Tauri) APIs to persist schedules securely and enable a future migration of the scheduler execution into the Tauri backend. This gives a fast, low-risk win (secure schedule persistence + UI wiring) while leaving the larger, production-grade backend scheduler as a planned follow-up.

The following tasks implement that recommendation. They are split into a small, fast path (persist & wire the UI) and a longer-term migration path (move execution to Tauri).

- [x] T090 [US2] Implement Tauri schedule persistence commands `get_schedule` and `set_schedule` in `src-tauri/src/commands.rs` and wire them to the existing storage backend (`src-tauri/src/storage.rs`). These commands should accept/return a JSON string representing the schedule config. ✅
- [x] T091 [US2] Add schedule persistence wrappers in `src/services/storage-service.ts`: `saveSchedule(schedule)` / `loadSchedule()` that call the new Tauri commands with a `localStorage` fallback. ✅
- [x] T092 [US2] Update `src/components/ScheduleConfig.tsx` to use `saveSchedule` / `loadSchedule` so the UI persists to secure storage, and add a small success/failure notification in the component. ✅
- [x] T093 [US2] Integrate `ScheduleConfig` into `App.tsx` (mount the component behind a menu or settings route) so users can edit and save schedules (this completes T046 as the UI is wired to persistent storage). ✅

Longer-term (recommended migration - lower priority but production-ready):

- [x] T094 [US2] Implement a backend scheduler in Tauri: add `start_scheduler` / `stop_scheduler` commands and an in-process scheduler that reads persisted schedules and performs clock-in/out operations. ✅
- [x] T095 [US2] Add eventing and notifications from Tauri to the renderer: expose status events (e.g., `scheduler.started`, `scheduler.clock_in.succeeded`, `scheduler.clock_out.failed`) and add native notifications and persistent logs in `src-tauri`. ✅

**Phase 4.5: Scheduler Architecture Consolidation (Priority: P1)**

Following successful completion of T094/T095, consolidate dual scheduler implementations to reduce complexity and improve maintainability.

- [x] T097 [US2] Simplify App.tsx navigation: consolidate "Backend Scheduler" and "Auto Schedule" tabs into unified "📅 Auto Schedule" tab that combines schedule configuration with backend scheduler controls. ✅
- [x] T099 [US2] Deprecate frontend ScheduleManager: mark `src/services/schedule-service.ts` as deprecated and update components to use backend scheduler exclusively via Tauri commands. ✅
- [x] T101 [US2] Clean up redundant code: remove unused frontend scheduling logic, duplicate type definitions, and obsolete timer-based implementations while preserving manual clock functionality. ✅
- [ ] T102 [US2] Integration testing: verify end-to-end backend scheduler functionality including schedule persistence, automatic execution, event emission, and manual override capabilities.

**Phase 4.6: Simplified Auto-Flow Implementation (Priority: P1)**

Based on updated requirements for ultra-simple user experience: setup once, then fully automatic daily operation. **Priority: Remove advanced mode and move auto-startup to backend.**

- [x] T104 [Simplify] Create simplified main status UI: replace complex tabs with single status screen showing current state and countdown timer. ✅
- [x] T105 [Simplify] Hide schedule configuration: move existing schedule UI to advanced/settings section, focus main UI on status display. ✅
- [x] T106 [Simplify] Implement app startup auto-clock-in logic: check attendance status on launch and automatically clock in if eligible (not same day, not rest day, not on leave). ✅
- [x] T107 [Backend] Remove advanced mode completely: eliminate advanced mode toggle from App.tsx and simplify to single status screen with emergency manual override only. ✅
- [x] T108 [Backend] Move auto clock-in to backend: migrate auto-startup logic from frontend useAutoStartup hook to backend scheduler initialization for better reliability. ✅
- [x] T109 [Backend] Update StatusScreen for pure display: remove auto-startup integration from frontend component, make it display-only with emergency manual controls. ✅

**Phase 4.6.1: Token Management & Skip Setup (Priority: P1)**

Based on user feedback for seamless token management and setup skip functionality.

- [x] T109.1 [Auth] Implement token-aware app routing: check for stored refresh_token on startup and skip setup screen if found. ✅
- [ ] T109.2 [Auth] Add automatic token refresh during clock operations: refresh tokens before each API call and save new tokens.
- [ ] T109.3 [Auth] Update backend auto clock-in to include token refresh: ensure tokens are refreshed and saved during automatic operations.
- [ ] T109.4 [Auth] Add token refresh to gap detection: when wake detection triggers auto clock-in, refresh and save tokens.
- [ ] T109.5 [Frontend] Update App.tsx routing logic: implement token check to determine whether to show setup or status screen.

**Phase 4.6.2: Storage-First Token Management Implementation (Priority: P1)**

Based on plan requirements for consistent token refresh across all operations and the scenario: "Clock-in check → If fails, refresh token → Save tokens → Retry once".

- [ ] T109.6 [Backend] Modify scheduler structure: Add AppHandle parameter to scheduler API methods (call_clock_in_api, call_clock_out_api, check_auto_startup) to enable storage access for token refresh operations.
- [ ] T109.7 [Backend] Implement storage-first token refresh in scheduler: Replace in-memory token logic with storage-based pattern matching api_manual_clock_in around line 380 in commands.rs. Each scheduler operation should: get refresh_token from storage → exchange for new tokens → store new tokens → use fresh token.
- [ ] T109.8 [Backend] Add comprehensive retry logic with double token refresh: Implement scenario-compliant retry pattern: 1st attempt with fresh token → if fails, refresh again → save tokens → retry once → if still fails, return error. This covers the "refresh token → save → retry once" requirement.
- [ ] T109.9 [Backend] Update attendance API with storage-first pattern: Apply same storage-based token refresh to get_attendance_status_api calls in scheduler, ensuring attendance checking uses fresh tokens with retry logic.
- [ ] T109.10 [Backend] Remove in-memory token management: Clean up scheduler by removing access_token Arc<Mutex<Option<String>>>, set_access_token method, and all in-memory token references to eliminate dual token management.
- [ ] T109.11 [Backend] Update scheduler command interfaces: Modify scheduler commands in commands.rs to pass AppHandle to scheduler methods and remove scheduler.set_access_token calls from manual API commands.
- [ ] T109.12 [Backend] Update background monitoring for storage-first: Remove scheduler.set_access_token calls from initialize_background_monitoring and ensure gap detection uses storage-based token refresh.
- [ ] T109.13 [Backend] Set 9-hour default for automatic operations: Update calculate_expected_clock_out_time to default to 540 minutes (9 hours) for automatic clock-out operations while preserving manual duration controls.
- [ ] T109.14 [Integration] Test storage-first token management: Verify all operations (manual commands, scheduler operations, attendance checking, gap detection) use consistent storage-first pattern with proper token refresh and retry logic.

**Implementation Notes:**

- Tasks T109.6-T109.14 implement the storage-first approach where ALL operations follow the same pattern: storage.retrieve("refresh_token") → exchange_refresh_token_api → storage.store(new_tokens) → API_call
- This eliminates the dual token management issue and ensures consistent behavior between manual commands and scheduler operations
- The retry logic implements the exact scenario: "If clock-in fails → Refresh token → Save new tokens → Retry once → If retry fails → Do nothing"
- These tasks align with plan.md requirement: "Token refresh: Automatic during all API operations, save new tokens"

**Later: Tray Integration (Phase 4.7)**

- [ ] T110 [Tray] Implement system tray integration: app runs in background, minimize to tray instead of exit, tray icon with status indicator and context menu.
- [ ] T111 [Tray] Window management: close button minimizes to tray, true exit only via tray context menu, restore window on tray click.
- [ ] T112 [Tray] System startup integration: optional auto-launch with Windows startup for truly set-and-forget operation.
- [ ] T113 [Tray] Final UI polish: streamline interface to show only essential status information and emergency manual controls.

Notes:

- The T090–T093 path is low-risk and quick: it only touches the storage surface and renderer wiring. It is sufficient to mark schedules as persisted and visible to users immediately.
- The T094–T095 path is recommended for production: moving timers and network side-effects to the backend improves reliability (survives renderer reloads), allows native notifications, and centralizes retry/backoff and logging.

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - manual operations + automation

---

## Phase 5: User Story 3 - Date-Based Logging and History (Priority: P2)

**Goal**: User can view organized logs of all clock activities, skipped days, and errors

**Independent Test**: After several clock operations, open logs and see chronological history with clear categories

### Implementation for User Story 3

- [ ] T047 [P] [US3] Create LogsViewer component in src/components/LogsViewer.tsx for log display and filtering
- [ ] T048 [US3] Implement logging service for date-organized activity tracking
- [ ] T049 [US3] Create log storage schema using client-side storage (localStorage/Tauri filesystem)
- [ ] T050 [US3] Add log entry creation for all clock operations (success/failure)
- [ ] T051 [US3] Add log categorization (clock-in, clock-out, skip, error) with timestamps
- [ ] T052 [US3] Implement log filtering by date range and event type
- [ ] T053 [US3] Add log export functionality (JSON/CSV) for user data portability
- [ ] T054 [US3] Create log pagination for large datasets (>1000 entries per PR-003)
- [ ] T055 [US3] Integrate logging into existing clock and schedule services
- [ ] T056 [US3] Add search functionality for log entries
- [ ] T057 [US3] Add JSDoc documentation to all US3 functions and components
- [ ] T058 [US3] Update App.tsx to integrate LogsViewer component

**Checkpoint**: All core functionality complete - manual, automatic, and logging systems work independently

---

## Phase 6: User Story 4 - API-Based Rest Day and Leave Detection (Priority: P3)

**Goal**: System automatically detects rest days and leave days via EMAPTA attendance API before clock operations

**Independent Test**: System checks attendance API, detects "Rest Day" or "On leave" status, skips operations and logs appropriately

### Implementation for User Story 4

- [ ] T059 [US4] Extend API client to support EMAPTA attendance status endpoint
- [ ] T060 [US4] Create attendance status checking function with API response parsing
- [ ] T061 [US4] Add attendance status interfaces in src/types/api.ts for EMAPTA response format
- [ ] T062 [US4] Integrate attendance checking into schedule service before clock operations
- [ ] T063 [US4] Add skip logic for "Rest Day" and "On leave" status detection
- [ ] T064 [US4] Add detailed logging for skipped days with leave/rest day reasons
- [ ] T065 [US4] Handle API unavailability gracefully (proceed with normal operations)
- [ ] T066 [US4] Add manual override for rest days with user confirmation
- [ ] T067 [US4] Add JSDoc documentation to all US4 functions
- [ ] T068 [US4] Update logging service to include skip reasons from API

**Checkpoint**: Smart skipping system functional - detects rest/leave days automatically

---

## Phase 7: User Story 5 - Timezone Handling and Mobile Interface (Priority: P3)

**Goal**: Mobile-friendly interface with proper timezone handling for accurate scheduling

**Independent Test**: Use app on mobile device, travel to different timezone, see correct schedule adjustment

### Implementation for User Story 5

- [ ] T069 [P] [US5] Add responsive CSS/styling for mobile-friendly interface (320px+ width per SC-005)
- [ ] T070 [P] [US5] Implement automatic timezone detection using date-fns and browser APIs
- [ ] T071 [US5] Add timezone preference settings in configuration
- [ ] T072 [US5] Update all time displays to show local timezone with UTC reference
- [ ] T073 [US5] Handle daylight saving time transitions in scheduling logic
- [ ] T074 [US5] Add touch-friendly controls for mobile devices
- [ ] T075 [US5] Create SettingsPanel component in src/components/SettingsPanel.tsx for preferences
- [ ] T076 [US5] Add timezone conversion utilities for accurate scheduling
- [ ] T077 [US5] Test and fix mobile interface responsiveness
- [ ] T078 [US5] Add JSDoc documentation to all US5 functions and components
- [ ] T079 [US5] Update App.tsx to integrate SettingsPanel component

**Checkpoint**: All user stories complete - full-featured clock automation system

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements and documentation

- [ ] T080 [P] Create comprehensive API integration documentation in docs/api-integration.md
- [ ] T081 [P] Create deployment and build guide in docs/deployment.md
- [ ] T082 [P] Create troubleshooting guide in docs/troubleshooting.md
- [ ] T083 [P] Add application icons for different platforms in public/icons/
- [ ] T084 Performance optimization - ensure <3s startup time (PR-001)
- [ ] T085 Security audit - verify token encryption and secure storage
- [ ] T086 [P] Code cleanup and consistent formatting with Biome
- [ ] T087 [P] Final JSDoc documentation review for completeness
- [ ] T088 Create user manual/quickstart guide
- [ ] T089 Final constitutional compliance check (TypeScript, Biome, naming conventions)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - User stories can proceed in parallel (if staffed) OR sequentially by priority
  - P1 (US1) → P2 (US2, US3) → P3 (US4, US5)
- **Polish (Phase 8)**: Depends on desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - Uses US1's auth/clock services but independently testable
- **User Story 3 (P2)**: Can start after Foundational - Logs US1/US2 operations but independently testable
- **User Story 4 (P3)**: Can start after Foundational - Enhances US2's scheduling but independently testable
- **User Story 5 (P3)**: Can start after Foundational - Improves all UIs but independently testable

### Within Each User Story

- Components marked [P] can be developed in parallel (different files)
- Services must be completed before components that depend on them
- Hooks created after services they wrap
- Integration tasks after individual components are complete
- JSDoc documentation concurrent with implementation

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational completes, user stories can start in parallel (if team capacity allows)
- Within each story, [P] tasks can run simultaneously
- Polish tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch components in parallel:
Task T024: "Create TokenSetup component in src/components/TokenSetup.tsx"
Task T025: "Create ClockControls component in src/components/ClockControls.tsx"

# Then services (may depend on types from foundation):
Task T026: "Implement authentication service in src/services/auth-service.ts"
Task T027: "Implement clock service in src/services/clock-service.ts"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test token setup and manual clock operations
5. Deploy/demo if ready - immediate value for manual clock management

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → Manual token + clock operations (MVP!)
3. Add User Story 2 → Automatic scheduling with 9-hour rule
4. Add User Story 3 → Complete activity logging and history
5. Add User Story 4 → Smart rest/leave day detection
6. Add User Story 5 → Mobile-friendly with timezone handling
7. Each story adds value without breaking previous functionality

### Parallel Team Strategy

With multiple developers after Foundational phase completion:

- Developer A: User Story 1 (MVP foundation)
- Developer B: User Story 2 (automation)
- Developer C: User Story 3 (logging)
- Stories integrate naturally due to well-defined interfaces

---

## Notes

- [P] tasks target different files with no dependencies
- [Story] labels map tasks to specific user stories for traceability
- Each user story independently completable and testable per Spec Kit workflow
- Constitutional compliance verified throughout (TypeScript, Biome, JSDoc, naming)
- Performance targets: <3s startup, <10s API operations, 95% automation success
- Security: Token encryption, HTTPS-only, rate limiting per requirements
- Client-side storage only (localStorage/Tauri filesystem) per constitution

# Black Bird - EMAPTA Clock Automation

Desktop application for automated EMAPTA clock-in/clock-out operations. Built with Tauri (Rust) + React + TypeScript.

## 🚀 Quick Start

```bash
npm install
npm run tauri dev
```

## 📊 Development Status

Get quick project context for development sessions:

```bash
npm run status
```

This shows current branch, task progress, and suggests specific files to examine for Claude Code sessions.

## 🤖 Claude Code Context

For efficient Claude Code sessions, use this context approach:

1. **Quick Status**: Run `npm run status` to see current state
2. **Read Key Files**: Start with `PROJECT_STATUS.md` then examine suggested files
3. **Update Notes**: Keep `DEVELOPMENT_NOTES.md` updated with progress

### Context Files
- `PROJECT_STATUS.md` - Current project state and priorities
- `DEVELOPMENT_NOTES.md` - Session notes and active work
- `.specify/specs/001-emapta-clock-automation/tasks.md` - Detailed task tracking

### Key Implementation Files
- `src/App.tsx` - Main UI structure and routing
- `src-tauri/src/scheduler.rs` - Backend scheduler and automation
- `src-tauri/src/commands.rs` - API integration and token management

## 🏗️ Architecture

- **Frontend**: React + TypeScript (minimal UI)
- **Backend**: Tauri + Rust (comprehensive scheduler)
- **Storage**: Encrypted secure storage for tokens
- **API**: EMAPTA integration with automatic token refresh

## 🎯 Current Focus

**Storage-First Token Management** - Ensuring all operations use consistent token refresh pattern from secure storage.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

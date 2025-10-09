#!/usr/bin/env node

/**
 * Development Status Script
 * Provides quick project context for Claude Code sessions
 */

import { execSync } from 'node:child_process';
import { readFileSync, existsSync } from 'node:fs';

function getGitInfo() {
  try {
    const branch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
    const lastCommit = execSync('git log -1 --oneline', { encoding: 'utf8' }).trim();
    const status = execSync('git status --porcelain', { encoding: 'utf8' }).trim();

    return { branch, lastCommit, hasChanges: !!status };
  } catch {
    return { branch: 'unknown', lastCommit: 'unavailable', hasChanges: false };
  }
}

function getProjectStatus() {
  const statusFile = 'PROJECT_STATUS.md';
  if (existsSync(statusFile)) {
    const content = readFileSync(statusFile, 'utf8');
    const lines = content.split('\n');

    // Extract key info
    const currentFocus = lines.find(l => l.includes('🎯 Current Focus'))?.split('\n')[1] || 'Not specified';
    const lastUpdated = lines.find(l => l.includes('Last Updated:'))?.split('**')[2] || 'Unknown';

    return { currentFocus, lastUpdated, exists: true };
  }

  return { exists: false };
}

function getTaskProgress() {
  const tasksFile = '.specify/specs/001-emapta-clock-automation/tasks.md';
  if (existsSync(tasksFile)) {
    const content = readFileSync(tasksFile, 'utf8');
    const completed = (content.match(/- \[x\]/g) || []).length;
    const pending = (content.match(/- \[ \]/g) || []).length;
    const total = completed + pending;

    return { completed, pending, total, exists: true };
  }

  return { exists: false };
}

function main() {
  console.log('\n🔍 BLACK BIRD - DEVELOPMENT STATUS\n');

  // Git Information
  const git = getGitInfo();
  console.log(`📍 Branch: ${git.branch}`);
  console.log(`📝 Last Commit: ${git.lastCommit}`);
  console.log(`🔄 Working Tree: ${git.hasChanges ? 'Modified' : 'Clean'}\n`);

  // Project Status
  const status = getProjectStatus();
  if (status.exists) {
    console.log(`🎯 Current Focus: ${status.currentFocus.replace(/\*/g, '').trim()}`);
    console.log(`📅 Last Updated: ${status.lastUpdated}\n`);
  }

  // Task Progress
  const tasks = getTaskProgress();
  if (tasks.exists) {
    const percentage = Math.round((tasks.completed / tasks.total) * 100);
    console.log(`📋 Task Progress: ${tasks.completed}/${tasks.total} (${percentage}%)`);
    console.log(`   ✅ Completed: ${tasks.completed}`);
    console.log(`   ⏳ Pending: ${tasks.pending}\n`);
  }

  // Quick Context Files
  console.log('📚 Key Context Files:');
  console.log('   • PROJECT_STATUS.md - Current state summary');
  console.log('   • DEVELOPMENT_NOTES.md - Session notes and blockers');
  console.log('   • .specify/specs/001-emapta-clock-automation/tasks.md - Detailed tasks');
  console.log('   • src/App.tsx - Main UI structure');
  console.log('   • src-tauri/src/scheduler.rs - Backend scheduler');
  console.log('   • src-tauri/src/commands.rs - API and token management\n');

  // Suggested Claude Prompt
  console.log('🤖 Suggested Claude Context Prompt:');
  console.log(`"Please read PROJECT_STATUS.md and then examine these key files:
   - .specify/specs/001-emapta-clock-automation/tasks.md (focus on pending tasks)
   - src-tauri/src/scheduler.rs (lines 100-250 for storage-first pattern)
   - src-tauri/src/commands.rs (lines 370-450 for token management)"`);

  console.log('\n✨ Ready for development!\n');
}

main();
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { WorkSchedule } from "../types/schedule";

/**
 * Backend Scheduler Service
 *
 * Provides integration with the Tauri backend scheduler for automatic
 * clock-in/out operations with event-driven status updates.
 */

// Types for scheduler events from backend
export interface SchedulerEvent {
  started?: { schedule: WorkSchedule };
  stopped?: {};
  clock_in_succeeded?: { operation_id: string; actual_time: string };
  clock_out_succeeded?: { operation_id: string; actual_time: string };
  clock_in_failed?: { operation_id: string; error: string };
  clock_out_failed?: { operation_id: string; error: string };
  error?: { message: string };
}

export interface SchedulerState {
  isRunning: boolean;
  currentSession: {
    clockedIn: boolean;
    clockInTime?: string;
    expectedClockOutTime?: string;
  };
  pendingOperations: Array<{
    id: string;
    operationType: "ClockIn" | "ClockOut";
    scheduledTime: string;
    status: string;
    actualTime?: string;
    errorMessage?: string;
  }>;
}

/**
 * Backend scheduler service class
 */
export class BackendSchedulerService {
  private eventListeners: Map<string, (event: SchedulerEvent) => void> =
    new Map();
  private unlistenFunction?: () => void;

  constructor() {
    this.setupEventListener();
  }

  /**
   * Set up event listener for scheduler events from backend
   */
  private async setupEventListener() {
    try {
      this.unlistenFunction = await listen<SchedulerEvent>(
        "scheduler_event",
        (event) => {
          console.log("[Backend Scheduler Event]", event.payload);

          // Notify all registered listeners
          this.eventListeners.forEach((callback) => {
            callback(event.payload);
          });
        }
      );
    } catch (error) {
      console.error("Failed to setup scheduler event listener:", error);
    }
  }

  /**
   * Register an event listener for scheduler events
   */
  addEventListener(id: string, callback: (event: SchedulerEvent) => void) {
    this.eventListeners.set(id, callback);
  }

  /**
   * Remove an event listener
   */
  removeEventListener(id: string) {
    this.eventListeners.delete(id);
  }

  /**
   * Start the backend scheduler with a work schedule
   */
  async startScheduler(schedule: WorkSchedule): Promise<void> {
    try {
      await invoke("start_scheduler", { schedule });
      console.log("[Backend Scheduler] Started with schedule:", schedule);
    } catch (error) {
      console.error("[Backend Scheduler] Failed to start:", error);
      throw new Error(`Failed to start scheduler: ${error}`);
    }
  }

  /**
   * Stop the backend scheduler
   */
  async stopScheduler(): Promise<void> {
    try {
      await invoke("stop_scheduler");
      console.log("[Backend Scheduler] Stopped");
    } catch (error) {
      console.error("[Backend Scheduler] Failed to stop:", error);
      throw new Error(`Failed to stop scheduler: ${error}`);
    }
  }

  /**
   * Get current scheduler state from backend
   */
  async getSchedulerState(): Promise<SchedulerState> {
    try {
      const state = await invoke<SchedulerState>("get_scheduler_state");
      return state;
    } catch (error) {
      console.error("[Backend Scheduler] Failed to get state:", error);
      throw new Error(`Failed to get scheduler state: ${error}`);
    }
  }

  /**
   * Set access token for API calls
   */
  async setAccessToken(token: string): Promise<void> {
    try {
      await invoke("set_scheduler_access_token", { token });
      console.log("[Backend Scheduler] Access token set");
    } catch (error) {
      console.error("[Backend Scheduler] Failed to set access token:", error);
      throw new Error(`Failed to set access token: ${error}`);
    }
  }

  /**
   * Manual clock-in operation
   */
  async manualClockIn(): Promise<boolean> {
    try {
      const result = await invoke<boolean>("scheduler_manual_clock_in");
      console.log("[Backend Scheduler] Manual clock-in result:", result);
      return result;
    } catch (error) {
      console.error("[Backend Scheduler] Manual clock-in failed:", error);
      throw new Error(`Manual clock-in failed: ${error}`);
    }
  }

  /**
   * Manual clock-out operation
   */
  async manualClockOut(bypassMinimum: boolean = false): Promise<boolean> {
    try {
      const result = await invoke<boolean>("scheduler_manual_clock_out", {
        bypassMinimum,
      });
      console.log("[Backend Scheduler] Manual clock-out result:", result);
      return result;
    } catch (error) {
      console.error("[Backend Scheduler] Manual clock-out failed:", error);
      throw new Error(`Manual clock-out failed: ${error}`);
    }
  }

  /**
   * Check if user can clock out (minimum duration check)
   */
  async canClockOut(): Promise<boolean> {
    try {
      const result = await invoke<boolean>("scheduler_can_clock_out");
      return result;
    } catch (error) {
      console.error(
        "[Backend Scheduler] Failed to check can clock out:",
        error
      );
      return false;
    }
  }

  /**
   * Clean up event listeners
   */
  destroy() {
    this.eventListeners.clear();
    if (this.unlistenFunction) {
      this.unlistenFunction();
    }
  }
}

// Global backend scheduler service instance
let backendSchedulerService: BackendSchedulerService | null = null;

/**
 * Get the global backend scheduler service instance
 */
export function getBackendSchedulerService(): BackendSchedulerService {
  if (!backendSchedulerService) {
    backendSchedulerService = new BackendSchedulerService();
  }
  return backendSchedulerService;
}

import type {
  WorkSchedule,
  ScheduledOperation,
  ScheduleState,
} from "../types/schedule";
import { clockIn, clockOut } from "./clock-service";

/**
 * @deprecated This frontend timer-based scheduler is being replaced by the backend scheduler.
 *
 * Use the backend scheduler via Tauri commands instead:
 * - src/services/backend-scheduler-service.ts
 * - src/hooks/use-backend-schedule.ts
 *
 * This service will be removed in a future version once the backend scheduler
 * is fully integrated and tested.
 */

/**
 * Schedule service for managing automatic clock operations
 *
 * Handles scheduling logic, 9-hour minimum enforcement, and automatic
 * clock-in/out operations based on user configuration.
 */

/**
 * Schedule manager class for handling all scheduling operations
 */
export class ScheduleManager {
  private schedule: WorkSchedule;
  private state: ScheduleState;
  private timers: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private accessToken: string | null = null;

  constructor(schedule: WorkSchedule, accessToken?: string | null) {
    this.schedule = schedule;
    this.accessToken = accessToken || null;
    this.state = {
      currentSession: {
        clockedIn: false,
      },
      pendingOperations: [],
    };
  }

  /**
   * Update access token for API calls
   */
  setAccessToken(accessToken: string | null): void {
    this.accessToken = accessToken;
  }

  /**
   * Update schedule configuration
   */
  updateSchedule(schedule: WorkSchedule): void {
    this.schedule = schedule;
    this.rescheduleOperations();
  }

  /**
   * Get current schedule state
   */
  getState(): ScheduleState {
    return { ...this.state };
  }

  /**
   * Start automatic scheduling based on configuration
   */
  async startAutoScheduling(): Promise<void> {
    if (!this.schedule.autoScheduleEnabled) {
      return;
    }

    // Clear existing timers
    this.clearAllTimers();

    // Schedule today's operations if not already clocked in
    await this.scheduleNextClockIn();
  }

  /**
   * Stop automatic scheduling
   */
  stopAutoScheduling(): void {
    this.clearAllTimers();
    this.state.pendingOperations = [];
  }

  /**
   * Manually clock in (bypasses scheduling)
   */
  async manualClockIn(): Promise<boolean> {
    try {
      const result = await clockIn(this.accessToken);
      if (result.success) {
        this.state.currentSession = {
          clockedIn: true,
          clockInTime: new Date().toISOString(),
          expectedClockOutTime: this.calculateClockOutTime(new Date()),
        };
        // Cancel pending clock-in operations
        this.cancelPendingOperations("clock-in");
        // Schedule clock-out
        await this.scheduleClockOut();
      }
      return result.success;
    } catch (error) {
      console.error("Manual clock-in failed:", error);
      return false;
    }
  }

  /**
   * Manually clock out (bypasses 9-hour minimum if user confirms)
   */
  async manualClockOut(bypassMinimum: boolean = false): Promise<boolean> {
    try {
      if (!bypassMinimum && !this.canClockOut()) {
        throw new Error("Cannot clock out before minimum work duration");
      }

      const result = await clockOut(this.accessToken);
      if (result.success) {
        this.state.currentSession = {
          clockedIn: false,
        };
        // Cancel pending clock-out operations
        this.cancelPendingOperations("clock-out");
        // Schedule next clock-in
        await this.scheduleNextClockIn();
      }
      return result.success;
    } catch (error) {
      console.error("Manual clock-out failed:", error);
      return false;
    }
  }

  /**
   * Check if user can clock out (9-hour minimum enforced)
   */
  canClockOut(): boolean {
    if (
      !this.state.currentSession.clockedIn ||
      !this.state.currentSession.clockInTime
    ) {
      return false;
    }

    const clockInTime = new Date(this.state.currentSession.clockInTime);
    const now = new Date();
    const elapsedMinutes =
      (now.getTime() - clockInTime.getTime()) / (1000 * 60);

    return elapsedMinutes >= this.schedule.minWorkDurationMinutes;
  }

  /**
   * Get time remaining until clock-out is allowed
   */
  getTimeUntilClockOut(): number {
    if (
      !this.state.currentSession.clockedIn ||
      !this.state.currentSession.clockInTime
    ) {
      return 0;
    }

    const clockInTime = new Date(this.state.currentSession.clockInTime);
    const now = new Date();
    const elapsedMinutes =
      (now.getTime() - clockInTime.getTime()) / (1000 * 60);
    const remainingMinutes =
      this.schedule.minWorkDurationMinutes - elapsedMinutes;

    return Math.max(0, remainingMinutes);
  }

  /**
   * Schedule next clock-in operation
   */
  private async scheduleNextClockIn(): Promise<void> {
    if (!this.schedule.autoScheduleEnabled || !this.schedule.clockInTime) {
      return;
    }

    const nextClockInTime = this.getNextClockInTime();
    const operation: ScheduledOperation = {
      id: `clock-in-${nextClockInTime.getTime()}`,
      type: "clock-in",
      scheduledTime: nextClockInTime.toISOString(),
      status: "pending",
    };

    this.state.pendingOperations.push(operation);

    const delay = nextClockInTime.getTime() - Date.now();
    const timer = setTimeout(() => {
      this.executeClockIn(operation.id);
    }, delay);

    this.timers.set(operation.id, timer);
  }

  /**
   * Schedule clock-out operation
   */
  private async scheduleClockOut(): Promise<void> {
    if (!this.state.currentSession.clockInTime) {
      return;
    }

    const clockOutTime = new Date(
      this.state.currentSession.expectedClockOutTime!
    );
    const operation: ScheduledOperation = {
      id: `clock-out-${clockOutTime.getTime()}`,
      type: "clock-out",
      scheduledTime: clockOutTime.toISOString(),
      status: "pending",
    };

    this.state.pendingOperations.push(operation);

    const delay = clockOutTime.getTime() - Date.now();
    const timer = setTimeout(() => {
      this.executeClockOut(operation.id);
    }, delay);

    this.timers.set(operation.id, timer);
  }

  /**
   * Execute automatic clock-in operation
   */
  private async executeClockIn(operationId: string): Promise<void> {
    const operation = this.state.pendingOperations.find(
      (op) => op.id === operationId
    );
    if (!operation) return;

    try {
      const result = await clockIn(this.accessToken);
      operation.status = result.success ? "completed" : "failed";
      operation.actualTime = new Date().toISOString();

      if (result.success) {
        this.state.currentSession = {
          clockedIn: true,
          clockInTime: operation.actualTime,
          expectedClockOutTime: this.calculateClockOutTime(
            new Date(operation.actualTime)
          ),
        };
        await this.scheduleClockOut();
      } else {
        operation.errorMessage = result.message || "Clock-in failed";
      }
    } catch (error) {
      operation.status = "failed";
      operation.errorMessage =
        error instanceof Error ? error.message : "Unknown error";
    }

    this.timers.delete(operationId);
  }

  /**
   * Execute automatic clock-out operation
   */
  private async executeClockOut(operationId: string): Promise<void> {
    const operation = this.state.pendingOperations.find(
      (op) => op.id === operationId
    );
    if (!operation) return;

    try {
      const result = await clockOut(this.accessToken);
      operation.status = result.success ? "completed" : "failed";
      operation.actualTime = new Date().toISOString();

      if (result.success) {
        this.state.currentSession = {
          clockedIn: false,
        };
        await this.scheduleNextClockIn();
      } else {
        operation.errorMessage = result.message || "Clock-out failed";
      }
    } catch (error) {
      operation.status = "failed";
      operation.errorMessage =
        error instanceof Error ? error.message : "Unknown error";
    }

    this.timers.delete(operationId);
  }

  /**
   * Calculate next clock-in time based on schedule
   */
  private getNextClockInTime(): Date {
    const now = new Date();
    const [hours, minutes] = this.schedule.clockInTime.split(":").map(Number);

    const nextClockIn = new Date(now);
    nextClockIn.setHours(hours, minutes, 0, 0);

    // If time has passed today, schedule for tomorrow
    if (nextClockIn <= now) {
      nextClockIn.setDate(nextClockIn.getDate() + 1);
    }

    return nextClockIn;
  }

  /**
   * Calculate clock-out time based on clock-in time and minimum duration
   */
  private calculateClockOutTime(clockInTime: Date): string {
    const clockOutTime = new Date(clockInTime);
    clockOutTime.setMinutes(
      clockOutTime.getMinutes() + this.schedule.minWorkDurationMinutes
    );
    return clockOutTime.toISOString();
  }

  /**
   * Cancel pending operations of specific type
   */
  private cancelPendingOperations(type: "clock-in" | "clock-out"): void {
    this.state.pendingOperations.forEach((operation) => {
      if (operation.type === type && operation.status === "pending") {
        operation.status = "cancelled";
        const timer = this.timers.get(operation.id);
        if (timer) {
          clearTimeout(timer);
          this.timers.delete(operation.id);
        }
      }
    });
  }

  /**
   * Reschedule operations when configuration changes
   */
  private rescheduleOperations(): void {
    this.clearAllTimers();
    this.state.pendingOperations = [];

    if (this.schedule.autoScheduleEnabled) {
      this.startAutoScheduling();
    }
  }

  /**
   * Clear all active timers
   */
  private clearAllTimers(): void {
    this.timers.forEach((timer) => clearTimeout(timer));
    this.timers.clear();
  }
}

// Global schedule manager instance
let scheduleManager: ScheduleManager | null = null;

/**
 * Initialize schedule manager with configuration
 */
export const initializeScheduleManager = (
  schedule: WorkSchedule,
  accessToken?: string | null
): ScheduleManager => {
  scheduleManager = new ScheduleManager(schedule, accessToken);
  return scheduleManager;
};

/**
 * Get current schedule manager instance
 */
export const getScheduleManager = (): ScheduleManager | null => {
  return scheduleManager;
};

/**
 * Default schedule configuration
 */
export const getDefaultSchedule = (): WorkSchedule => ({
  clockInTime: "09:00",
  timezone: "America/New_York",
  autoScheduleEnabled: false,
  minWorkDurationMinutes: 550, // 9 hours 10 minutes
});

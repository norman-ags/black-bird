/**
 * Work schedule configuration
 */
export interface WorkSchedule {
  /** Preferred clock-in time in HH:MM format */
  clockInTime: string;
  /** Timezone preference (e.g., 'America/New_York') */
  timezone: string;
  /** Whether automatic scheduling is enabled */
  autoScheduleEnabled: boolean;
  /** Minimum work duration in minutes (default: 540 = 9 hours) */
  minWorkDurationMinutes: number;
}

/**
 * Scheduled clock operation
 */
export interface ScheduledOperation {
  id: string;
  type: "clock-in" | "clock-out";
  scheduledTime: string; // ISO timestamp
  status: "pending" | "completed" | "failed" | "cancelled";
  actualTime?: string; // ISO timestamp when completed
  errorMessage?: string;
}

/**
 * Schedule state for tracking current work session
 */
export interface ScheduleState {
  currentSession: {
    clockedIn: boolean;
    clockInTime?: string; // ISO timestamp
    expectedClockOutTime?: string; // ISO timestamp
  };
  pendingOperations: ScheduledOperation[];
}

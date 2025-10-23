import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { WorkSchedule } from "../types/schedule";
import {
  getBackendSchedulerService,
  type SchedulerEvent,
  type SchedulerState,
} from "../services/backend-scheduler-service";
import { useAuth } from "./use-auth";

// Define AttendanceItem type based on the API response
interface AttendanceItem {
  work_date: string;
  attendance_status: string;
  date_time_in: string | null;
  date_time_out: string | null;
  is_restday: boolean | null;
}

/**
 * Custom hook for backend scheduler integration
 *
 * Provides integration with the Tauri backend scheduler for automatic
 * clock operations with real-time event updates.
 */

interface UseBackendScheduleReturn {
  // Schedule configuration
  schedule: WorkSchedule | null;

  // Scheduler state
  schedulerState: SchedulerState | null;
  isSchedulerRunning: boolean;
  canClockOut: boolean;

  // Manual operations
  manualClockIn: () => Promise<boolean>;
  manualClockOut: (bypassMinimum?: boolean) => Promise<boolean>;
  syncWithRealAttendance: () => Promise<void>;

  // Schedule control
  startScheduler: (schedule: WorkSchedule) => Promise<void>;
  stopScheduler: () => Promise<void>;

  // Loading and error states
  isLoading: boolean;
  error: string | null;
}

export const useBackendSchedule = (): UseBackendScheduleReturn => {
  const { accessToken } = useAuth();

  // State
  const [schedule, setSchedule] = useState<WorkSchedule | null>(null);
  const [schedulerState, setSchedulerState] = useState<SchedulerState | null>(
    null
  );
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [canClockOut, setCanClockOut] = useState(false);

  // Service instance
  const serviceRef = useRef<ReturnType<
    typeof getBackendSchedulerService
  > | null>(null);

  /**
   * Initialize backend scheduler service
   */
  // useEffect(() => {
  //   try {
  //     serviceRef.current = getBackendSchedulerService();

  //     // Set up event listener for scheduler events
  //     const eventListener = (event: SchedulerEvent) => {
  //       console.log("[useBackendSchedule] Received event:", event);

  //       if (event.started) {
  //         setSchedule(event.started.schedule);
  //         refreshSchedulerState();
  //       } else if (event.stopped) {
  //         refreshSchedulerState();
  //       } else if (event.clock_in_succeeded || event.clock_out_succeeded) {
  //         refreshSchedulerState();
  //         refreshCanClockOut();
  //       } else if (event.clock_in_failed || event.clock_out_failed) {
  //         const errorMsg =
  //           event.clock_in_failed?.error ||
  //           event.clock_out_failed?.error ||
  //           "Unknown error";
  //         setError(errorMsg);
  //         refreshSchedulerState();
  //       } else if (event.error) {
  //         setError(event.error.message);
  //       }
  //     };

  //     serviceRef.current.addEventListener("useBackendSchedule", eventListener);

  //     // Initial state load
  //     refreshSchedulerState();
  //     refreshCanClockOut();

  //     return () => {
  //       if (serviceRef.current) {
  //         serviceRef.current.removeEventListener("useBackendSchedule");
  //       }
  //     };
  //   } catch (err) {
  //     console.error("Failed to initialize backend scheduler service:", err);
  //     setError(
  //       err instanceof Error ? err.message : "Failed to initialize scheduler"
  //     );
  //   }
  // }, []);

  /**
   * Update access token when it changes
   */
  // useEffect(() => {
  //   if (!serviceRef.current || !accessToken) return;

  //   const updateToken = async () => {
  //     try {
  //       await serviceRef.current!.setAccessToken(accessToken);
  //     } catch (err) {
  //       console.error("Failed to set access token:", err);
  //       setError("Failed to set access token");
  //     }
  //   };

  //   updateToken();
  // }, [accessToken]);

  /**
   * Refresh scheduler state from backend
   */
  const refreshSchedulerState = useCallback(async () => {
    if (!serviceRef.current) return;

    try {
      const state = await serviceRef.current.getSchedulerState();
      setSchedulerState(state);
    } catch (err) {
      console.error("Failed to refresh scheduler state:", err);
    }
  }, []);

  /**
   * Refresh can clock out status
   */
  const refreshCanClockOut = useCallback(async () => {
    if (!serviceRef.current) return;

    try {
      const canClock = await serviceRef.current.canClockOut();
      setCanClockOut(canClock);
    } catch (err) {
      console.error("Failed to check can clock out:", err);
    }
  }, []);

  /**
   * Start the scheduler with a work schedule
   */
  const startScheduler = useCallback(
    async (workSchedule: WorkSchedule) => {
      if (!serviceRef.current) {
        throw new Error("Backend scheduler not available");
      }

      try {
        setIsLoading(true);
        setError(null);

        await serviceRef.current.startScheduler(workSchedule);
        setSchedule(workSchedule);

        // Refresh state after starting
        await refreshSchedulerState();
      } catch (err) {
        console.error("Failed to start scheduler:", err);
        const errorMsg =
          err instanceof Error ? err.message : "Failed to start scheduler";
        setError(errorMsg);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [refreshSchedulerState]
  );

  /**
   * Stop the scheduler
   */
  const stopScheduler = useCallback(async () => {
    if (!serviceRef.current) {
      throw new Error("Backend scheduler not available");
    }

    try {
      setIsLoading(true);
      setError(null);

      await serviceRef.current.stopScheduler();

      // Refresh state after stopping
      await refreshSchedulerState();
    } catch (err) {
      console.error("Failed to stop scheduler:", err);
      const errorMsg =
        err instanceof Error ? err.message : "Failed to stop scheduler";
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [refreshSchedulerState]);

  /**
   * Manual clock-in operation
   */
  const manualClockIn = useCallback(async (): Promise<boolean> => {
    if (!serviceRef.current) {
      throw new Error("Backend scheduler not available");
    }

    try {
      setIsLoading(true);
      setError(null);

      const result = await serviceRef.current.manualClockIn();

      // Refresh state and can clock out status
      await Promise.all([refreshSchedulerState(), refreshCanClockOut()]);

      return result;
    } catch (err) {
      console.error("Manual clock-in failed:", err);
      const errorMsg = err instanceof Error ? err.message : "Clock-in failed";
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [refreshSchedulerState, refreshCanClockOut]);

  /**
   * Manual clock-out operation
   */
  const manualClockOut = useCallback(
    async (bypassMinimum: boolean = false): Promise<boolean> => {
      if (!serviceRef.current) {
        throw new Error("Backend scheduler not available");
      }

      try {
        setIsLoading(true);
        setError(null);

        const result = await serviceRef.current.manualClockOut(bypassMinimum);

        // Refresh state and can clock out status
        await Promise.all([refreshSchedulerState(), refreshCanClockOut()]);

        return result;
      } catch (err) {
        console.error("Manual clock-out failed:", err);
        const errorMsg =
          err instanceof Error ? err.message : "Clock-out failed";
        setError(errorMsg);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [refreshSchedulerState, refreshCanClockOut]
  );

  /**
   * Sync local scheduler state with real EMAPTA attendance status
   */
  const syncWithRealAttendance = useCallback(async (): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);

      // Get real attendance status from EMAPTA API
      const attendanceStatus = await invoke<AttendanceItem | null>(
        "api_get_attendance_status"
      );

      if (attendanceStatus) {
        // Update scheduler state based on real attendance
        const isReallyClocked =
          attendanceStatus.attendance_status === "Started" &&
          attendanceStatus.date_time_in != null &&
          attendanceStatus.date_time_out == null;

        console.log(
          `Real attendance status: ${attendanceStatus.attendance_status}, Currently clocked in: ${isReallyClocked}`
        );

        // Update scheduler state to match reality
        if (schedulerState) {
          const updatedState = {
            ...schedulerState,
            currentSession: {
              ...schedulerState.currentSession,
              clockedIn: isReallyClocked,
              clockInTime: isReallyClocked
                ? attendanceStatus.date_time_in || undefined
                : undefined,
              expectedClockOutTime:
                isReallyClocked && attendanceStatus.date_time_in
                  ? new Date(
                      new Date(attendanceStatus.date_time_in).getTime() +
                        9 * 60 * 60 * 1000
                    ).toISOString()
                  : undefined,
            },
          };
          setSchedulerState(updatedState);
        }
      }

      // Also refresh the scheduler state
      await refreshSchedulerState();
    } catch (err) {
      console.error("Failed to sync with real attendance:", err);
      const errorMsg = err instanceof Error ? err.message : "Sync failed";
      setError(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }, [schedulerState, refreshSchedulerState]);

  /**
   * Sync with real attendance status when accessToken changes
   */
  // useEffect(() => {
  //   if (!accessToken) return;

  //   const syncAttendance = async () => {
  //     try {
  //       // Small delay to ensure everything is initialized
  //       setTimeout(async () => {
  //         await syncWithRealAttendance();
  //       }, 1500);
  //     } catch (err) {
  //       console.error("Failed to sync attendance on load:", err);
  //     }
  //   };

  //   syncAttendance();
  // }, [accessToken, syncWithRealAttendance]);

  // Derived state
  const isSchedulerRunning = schedulerState?.isRunning ?? false;

  return {
    // Schedule configuration
    schedule,

    // Scheduler state
    schedulerState,
    isSchedulerRunning,
    canClockOut,

    // Manual operations
    manualClockIn,
    manualClockOut,
    syncWithRealAttendance,

    // Schedule control
    startScheduler,
    stopScheduler,

    // Loading and error states
    isLoading,
    error,
  };
};

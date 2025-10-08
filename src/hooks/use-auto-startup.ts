import { useEffect, useRef } from "react";
import { useAuth } from "./use-auth";
import { useBackendSchedule } from "./use-backend-schedule";

/**
 * Auto Startup Hook
 *
 * Handles automatic clock-in logic when the app starts:
 * 1. Check if user is authenticated
 * 2. Check if already clocked in today
 * 3. Check if it's a work day (TODO: add attendance API integration)
 * 4. Automatically clock in if eligible
 */
export const useAutoStartup = () => {
  const { accessToken } = useAuth();
  const { schedulerState, manualClockIn, isLoading } = useBackendSchedule();
  const hasRun = useRef(false);

  useEffect(() => {
    // Only run once when component mounts
    if (hasRun.current) return;

    // Only run if authenticated
    if (!accessToken) return;

    // Don't run if already loading
    if (isLoading) return;

    const runAutoStartup = async () => {
      hasRun.current = true;

      try {
        console.log("[AutoStartup] Checking if auto clock-in should run...");

        // Check if already clocked in
        if (schedulerState?.currentSession?.clockedIn) {
          console.log(
            "[AutoStartup] Already clocked in, skipping auto clock-in"
          );
          return;
        }

        // Check if already clocked in today (by checking clock-in time)
        const today = new Date().toDateString();
        const lastClockIn = schedulerState?.currentSession?.clockInTime;
        if (lastClockIn) {
          const lastClockInDate = new Date(lastClockIn).toDateString();
          if (lastClockInDate === today) {
            console.log(
              "[AutoStartup] Already clocked in today, skipping auto clock-in"
            );
            return;
          }
        }

        // TODO: Add attendance API check for rest days/leave
        // For now, proceed with auto clock-in

        console.log(
          "[AutoStartup] Conditions met, attempting auto clock-in..."
        );

        // Attempt auto clock-in
        const success = await manualClockIn();

        if (success) {
          console.log("[AutoStartup] Auto clock-in successful!");
        } else {
          console.log("[AutoStartup] Auto clock-in failed");
        }
      } catch (error) {
        console.error("[AutoStartup] Error during auto startup:", error);
      }
    };

    // Run with a small delay to ensure everything is initialized
    const timeout = setTimeout(runAutoStartup, 1000);

    return () => clearTimeout(timeout);
  }, [
    accessToken,
    schedulerState?.currentSession?.clockedIn,
    schedulerState?.currentSession?.clockInTime,
    manualClockIn,
    isLoading,
  ]);

  return {
    hasAttemptedAutoStart: hasRun.current,
  };
};

import type React from "react";
import { useEffect, useRef } from "react";
import { useAuth } from "../hooks/use-auth";
import { useBackendSchedule } from "../hooks/use-backend-schedule";
import {
  useAttendanceQuery,
  isCurrentlyClocked,
  getClockInTime,
  getExpectedClockOutTime,
  hasCompletedShift,
  getCompletedShiftDetails,
} from "../queries/attendance";
import { invoke } from "@tauri-apps/api/core";

/**
 * Simplified Status Component
 *
 * Pure display component showing current clock status and countdown timer.
 * All automation logic is handled by the backend.
 */
export const StatusScreen: React.FC = () => {
  const { refreshToken } = useAuth();
  const { manualClockIn, manualClockOut, isLoading, error } =
    useBackendSchedule();

  // Use React Query to fetch real attendance status
  const { data: attendanceData } = useAttendanceQuery();

  // Get real clock status from EMAPTA API data
  const realIsCurrentlyClockedIn = isCurrentlyClocked(attendanceData ?? null);
  const realClockInTime = getClockInTime(attendanceData ?? null);
  const realExpectedClockOutTime = getExpectedClockOutTime(
    attendanceData ?? null
  );
  const shiftCompleted = hasCompletedShift(attendanceData ?? null);
  const completedShiftDetails = getCompletedShiftDetails(attendanceData ?? null);

  const hasInitializedMonitoring = useRef(false);

  // Initialize background monitoring when app starts
  // The AI keep insisting to run call this in the FE and keeps failing when he tried to run on BE
  // Ideally this should be running in the BE
  useEffect(() => {
    // Only run once when component mounts
    if (hasInitializedMonitoring.current) return;

    // Only run if authenticated
    if (!refreshToken) return;

    // Don't run if already loading
    if (isLoading) return;

    const initializeMonitoring = async () => {
      hasInitializedMonitoring.current = true;

      try {
        console.log("[StatusScreen] Initializing background monitoring...");
        await invoke<string>("initialize_background_monitoring");
        console.log(
          "[StatusScreen] Background monitoring initialized successfully"
        );
      } catch (error) {
        console.error(
          "[StatusScreen] Failed to initialize background monitoring:",
          error
        );
      }
    };

    // Run with a small delay to ensure everything is initialized
    const timeout = setTimeout(initializeMonitoring, 1000);

    return () => clearTimeout(timeout);
  }, [refreshToken, isLoading]);

  // If not authenticated, show setup prompt
  if (!refreshToken) {
    return (
      <div style={{ textAlign: "center", padding: "48px 24px" }}>
        <div style={{ marginBottom: "32px" }}>
          <h2>🔑 Welcome to Black Bird</h2>
          <p style={{ color: "#6b7280", marginBottom: "24px" }}>
            Set up your refresh token to get started with automatic clock
            management.
          </p>
        </div>

        <div
          style={{
            background: "#f9fafb",
            border: "1px solid #e5e7eb",
            borderRadius: "8px",
            padding: "24px",
            maxWidth: "400px",
            margin: "0 auto",
          }}
        >
          <p style={{ marginBottom: "16px", fontWeight: "500" }}>
            ⚡ After setup, the app will:
          </p>
          <ul
            style={{ textAlign: "left", color: "#6b7280", paddingLeft: "20px" }}
          >
            <li>Automatically clock you in when you open the app</li>
            <li>Automatically clock you out after 9 hours 10 minutes</li>
            <li>Skip weekends and holidays automatically</li>
            <li>Run in the background with minimal interaction</li>
          </ul>
        </div>
      </div>
    );
  }

  // Use real attendance data directly
  const isCurrentlyClockedIn = realIsCurrentlyClockedIn;
  const clockInTime = realClockInTime;
  const expectedClockOutTime = realExpectedClockOutTime;

  // Calculate time remaining until clock out
  const getTimeRemaining = () => {
    if (!isCurrentlyClockedIn || !expectedClockOutTime) return null;

    const now = new Date();
    const clockOutTime = new Date(expectedClockOutTime);
    const diffMs = clockOutTime.getTime() - now.getTime();

    if (diffMs <= 0) return "Ready to clock out";

    const hours = Math.floor(diffMs / (1000 * 60 * 60));
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    return `${hours}h ${minutes}m remaining`;
  };

  const timeRemaining = getTimeRemaining();

  return (
    <div style={{ maxWidth: "600px", margin: "0 auto", padding: "24px" }}>
      {/* Main Status Display */}
      <div style={{ textAlign: "center", marginBottom: "32px" }}>
        <h2 style={{ marginBottom: "16px" }}>
          {isCurrentlyClockedIn
            ? "🟢 Currently Working"
            : shiftCompleted
            ? "✅ Shift Completed"
            : "⚪ Not Clocked In"}
        </h2>

        {isCurrentlyClockedIn && clockInTime && (
          <div
            style={{
              background: "#f0f9ff",
              border: "2px solid #3b82f6",
              borderRadius: "12px",
              padding: "24px",
              marginBottom: "24px",
            }}
          >
            <div
              style={{
                fontSize: "18px",
                fontWeight: "600",
                marginBottom: "8px",
              }}
            >
              Clocked in at {new Date(clockInTime).toLocaleTimeString()}
            </div>
            {expectedClockOutTime && (
              <div style={{ color: "#6b7280", marginBottom: "8px" }}>
                Expected clock out:{" "}
                {new Date(expectedClockOutTime).toLocaleTimeString()}
              </div>
            )}
            {timeRemaining && (
              <div
                style={{
                  fontSize: "16px",
                  color: "#3b82f6",
                  fontWeight: "500",
                }}
              >
                {timeRemaining}
              </div>
            )}
          </div>
        )}

        {shiftCompleted && completedShiftDetails && (
          <div
            style={{
              background: "#f0fdf4",
              border: "2px solid #10b981",
              borderRadius: "12px",
              padding: "24px",
              marginBottom: "24px",
            }}
          >
            <div
              style={{
                fontSize: "18px",
                fontWeight: "600",
                marginBottom: "8px",
              }}
            >
              Work day completed successfully!
            </div>
            <div style={{ color: "#6b7280", marginBottom: "4px" }}>
              Clocked in: {new Date(completedShiftDetails.clockInTime).toLocaleTimeString()}
            </div>
            <div style={{ color: "#6b7280" }}>
              Clocked out: {new Date(completedShiftDetails.clockOutTime).toLocaleTimeString()}
            </div>
          </div>
        )}

        {!isCurrentlyClockedIn && !shiftCompleted && (
          <div
            style={{
              background: "#f9fafb",
              border: "1px solid #e5e7eb",
              borderRadius: "12px",
              padding: "24px",
              marginBottom: "24px",
            }}
          >
            <div style={{ fontSize: "16px", color: "#6b7280" }}>
              Ready to start your work day
            </div>
          </div>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div
          style={{
            background: "#fef2f2",
            border: "1px solid #fecaca",
            borderRadius: "8px",
            padding: "16px",
            marginBottom: "24px",
            color: "#991b1b",
          }}
        >
          <strong>⚠️ Error:</strong> {error}
        </div>
      )}

      {/* Manual Override Section */}
      <div
        style={{
          background: "#f9fafb",
          border: "1px solid #e5e7eb",
          borderRadius: "8px",
          padding: "20px",
        }}
      >
        <div style={{ marginBottom: "16px" }}>
          <h3 style={{ fontSize: "16px", fontWeight: "600", color: "#374151" }}>
            ⚠️ Manual Override (Emergency Use)
          </h3>
          <p style={{ fontSize: "14px", color: "#6b7280", margin: "4px 0" }}>
            Use these controls only in exceptional circumstances
          </p>
        </div>

        <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
          <button
            type="button"
            onClick={manualClockIn}
            disabled={isLoading || isCurrentlyClockedIn || shiftCompleted}
            style={{
              padding: "10px 20px",
              backgroundColor: (isCurrentlyClockedIn || shiftCompleted) ? "#d1d5db" : "#10b981",
              color: "white",
              border: "none",
              borderRadius: "6px",
              cursor: (isCurrentlyClockedIn || shiftCompleted) ? "not-allowed" : "pointer",
              fontSize: "14px",
              fontWeight: "500",
            }}
          >
            {isLoading ? "⏳ Processing..." : "🟢 Manual Clock In"}
          </button>

          <button
            type="button"
            onClick={() => manualClockOut()}
            disabled={isLoading || !isCurrentlyClockedIn || shiftCompleted}
            style={{
              padding: "10px 20px",
              backgroundColor: (!isCurrentlyClockedIn || shiftCompleted) ? "#d1d5db" : "#ef4444",
              color: "white",
              border: "none",
              borderRadius: "6px",
              cursor: (!isCurrentlyClockedIn || shiftCompleted) ? "not-allowed" : "pointer",
              fontSize: "14px",
              fontWeight: "500",
            }}
          >
            {isLoading ? "⏳ Processing..." : "🔴 Manual Clock Out"}
          </button>
        </div>
      </div>

      {/* Today's Summary */}
      <div
        style={{
          marginTop: "24px",
          textAlign: "center",
          fontSize: "14px",
          color: "#6b7280",
        }}
      >
        <div>
          📅 {new Date().toLocaleDateString()} • Status:{" "}
          {isCurrentlyClockedIn
            ? "Working"
            : shiftCompleted
            ? "Shift Completed"
            : "Available"}
        </div>
      </div>
    </div>
  );
};

export default StatusScreen;

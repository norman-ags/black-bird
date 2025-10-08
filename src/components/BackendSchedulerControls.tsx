import { useState } from "react";
import { useBackendSchedule } from "../hooks/use-backend-schedule";
import { useAuth } from "../hooks/use-auth";
import type { WorkSchedule } from "../types/schedule";

/**
 * BackendSchedulerControls component - provides backend scheduler management
 * Includes manual operations and automatic scheduling control
 */
export default function BackendSchedulerControls() {
  const { accessToken, refreshToken } = useAuth();
  const {
    schedulerState,
    isSchedulerRunning,
    canClockOut,
    manualClockIn,
    manualClockOut,
    startScheduler,
    stopScheduler,
    isLoading,
    error,
  } = useBackendSchedule();

  const [lastOperation, setLastOperation] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<"success" | "error" | "info">(
    "info"
  );

  // Test schedule for demonstration
  const testSchedule: WorkSchedule = {
    clockInTime: "09:00",
    timezone: "America/New_York",
    autoScheduleEnabled: true,
    minWorkDurationMinutes: 540, // 9 hours
  };

  /**
   * Handle manual clock in
   */
  async function handleClockIn() {
    if (!accessToken) {
      setMessage(
        "No access token available. Please validate your refresh token first."
      );
      setMessageType("error");
      return;
    }

    setMessage(null);
    try {
      const result = await manualClockIn();
      setLastOperation("Manual Clock In");
      setMessage(
        result ? "Successfully clocked in via backend" : "Clock in failed"
      );
      setMessageType(result ? "success" : "error");
    } catch (error: any) {
      const errorMessage = error.message || String(error);
      setMessage(`Clock in failed: ${errorMessage}`);
      setMessageType("error");
    }
  }

  /**
   * Handle manual clock out
   */
  async function handleClockOut() {
    if (!accessToken) {
      setMessage(
        "No access token available. Please validate your refresh token first."
      );
      setMessageType("error");
      return;
    }

    setMessage(null);
    try {
      let result: boolean;

      if (!canClockOut) {
        const shouldBypass = window.confirm(
          "You haven't reached the minimum work duration (9 hours). Clock out anyway?"
        );
        if (!shouldBypass) {
          setMessage("Clock out cancelled - minimum duration not reached");
          setMessageType("info");
          return;
        }
        result = await manualClockOut(true); // Bypass minimum
      } else {
        result = await manualClockOut(false);
      }

      setLastOperation("Manual Clock Out");
      setMessage(
        result ? "Successfully clocked out via backend" : "Clock out failed"
      );
      setMessageType(result ? "success" : "error");
    } catch (error: any) {
      const errorMessage = error.message || String(error);
      setMessage(`Clock out failed: ${errorMessage}`);
      setMessageType("error");
    }
  }

  /**
   * Handle start scheduler
   */
  async function handleStartScheduler() {
    setMessage(null);
    try {
      await startScheduler(testSchedule);
      setLastOperation("Start Scheduler");
      setMessage("Backend scheduler started successfully");
      setMessageType("success");
    } catch (error: any) {
      const errorMessage = error.message || String(error);
      setMessage(`Failed to start scheduler: ${errorMessage}`);
      setMessageType("error");
    }
  }

  /**
   * Handle stop scheduler
   */
  async function handleStopScheduler() {
    setMessage(null);
    try {
      await stopScheduler();
      setLastOperation("Stop Scheduler");
      setMessage("Backend scheduler stopped successfully");
      setMessageType("success");
    } catch (error: any) {
      const errorMessage = error.message || String(error);
      setMessage(`Failed to stop scheduler: ${errorMessage}`);
      setMessageType("error");
    }
  }

  // Show setup message if no refresh token
  if (!refreshToken) {
    return (
      <div className="backend-scheduler-controls">
        <h2>Backend Scheduler</h2>
        <p style={{ color: "orange" }}>
          Please set up your refresh token first to enable scheduler operations.
        </p>
      </div>
    );
  }

  return (
    <div className="backend-scheduler-controls">
      <h2>Backend Scheduler Controls</h2>

      {/* Scheduler Status */}
      <div
        style={{
          marginBottom: "20px",
          padding: "10px",
          border: "1px solid #ccc",
          borderRadius: "5px",
        }}
      >
        <h3>Scheduler Status</h3>
        <p>Running: {isSchedulerRunning ? "✅ Yes" : "❌ No"}</p>

        {schedulerState && (
          <div style={{ fontSize: "0.9em", color: "gray" }}>
            <p>
              Session Status:{" "}
              {schedulerState.currentSession.clockedIn
                ? "🟢 Clocked In"
                : "🔴 Clocked Out"}
            </p>

            {schedulerState.currentSession.clockInTime && (
              <p>
                Clock-in time:{" "}
                {new Date(
                  schedulerState.currentSession.clockInTime
                ).toLocaleString()}
              </p>
            )}

            {schedulerState.currentSession.expectedClockOutTime && (
              <p>
                Expected clock-out:{" "}
                {new Date(
                  schedulerState.currentSession.expectedClockOutTime
                ).toLocaleString()}
              </p>
            )}

            {schedulerState.pendingOperations.length > 0 && (
              <div>
                <p>Pending Operations:</p>
                <ul>
                  {schedulerState.pendingOperations.map((op) => (
                    <li key={op.id} style={{ fontSize: "0.8em" }}>
                      {op.operationType} at{" "}
                      {new Date(op.scheduledTime).toLocaleString()} ({op.status}
                      )
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        <p>Can Clock Out: {canClockOut ? "✅ Yes" : "❌ No (need 9 hours)"}</p>
      </div>

      {/* Manual Operations */}
      <div style={{ marginBottom: "20px" }}>
        <h3>Manual Operations</h3>
        <div style={{ marginBottom: "15px" }}>
          <button
            onClick={handleClockIn}
            disabled={isLoading}
            style={{ marginRight: "10px", padding: "10px 20px" }}
          >
            {isLoading ? "Processing..." : "Manual Clock In"}
          </button>

          <button
            onClick={handleClockOut}
            disabled={isLoading}
            style={{ padding: "10px 20px" }}
          >
            {isLoading ? "Processing..." : "Manual Clock Out"}
          </button>
        </div>
      </div>

      {/* Scheduler Control */}
      <div style={{ marginBottom: "20px" }}>
        <h3>Automatic Scheduler</h3>
        <div style={{ marginBottom: "15px" }}>
          <button
            onClick={handleStartScheduler}
            disabled={isLoading || isSchedulerRunning}
            style={{
              marginRight: "10px",
              padding: "10px 20px",
              backgroundColor: "green",
              color: "white",
            }}
          >
            {isLoading ? "Processing..." : "Start Scheduler"}
          </button>

          <button
            onClick={handleStopScheduler}
            disabled={isLoading || !isSchedulerRunning}
            style={{
              padding: "10px 20px",
              backgroundColor: "red",
              color: "white",
            }}
          >
            {isLoading ? "Processing..." : "Stop Scheduler"}
          </button>
        </div>

        <p style={{ fontSize: "0.9em", color: "gray" }}>
          Schedule: Clock in at 9:00 AM EST, work for 9 hours minimum
        </p>
      </div>

      {/* Status Messages */}
      {lastOperation && (
        <p style={{ color: "blue" }}>
          Last operation: {lastOperation} at {new Date().toLocaleTimeString()}
        </p>
      )}

      {message && (
        <p
          style={{
            color:
              messageType === "success"
                ? "green"
                : messageType === "error"
                ? "red"
                : "blue",
            marginTop: "10px",
          }}
        >
          {message}
        </p>
      )}

      {error && (
        <p style={{ color: "red", marginTop: "10px" }}>
          Backend Error: {error}
        </p>
      )}

      {accessToken && (
        <p style={{ fontSize: "0.9em", color: "gray", marginTop: "10px" }}>
          ✓ Access token configured for API calls
        </p>
      )}
    </div>
  );
}

import { useState } from "react";
import { useClock } from "../hooks/use-clock";
import { useAuth } from "../hooks/use-auth";
/**
 * ClockControls component - provides manual clock in/out operations
 * Requires valid access token from auth service
 */
export default function ClockControls() {
  const { accessToken, refreshToken, authenticate } = useAuth();
  const { busy, doClockIn, doClockOut } = useClock(accessToken);
  const [lastOperation, setLastOperation] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<"success" | "error" | "info">(
    "info"
  );

  /**
   * Handle clock in operation with error handling
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
      const result = await doClockIn();
      setLastOperation("Clock In");
      setMessage("Successfully clocked in");
      setMessageType("success");
      console.log("Clock in result:", result);
    } catch (error: any) {
      const errorMessage = error.message || String(error);
      setMessage(`Clock in failed: ${errorMessage}`);
      setMessageType("error");
      console.error("Clock in error:", error);
    }
  }

  /**
   * Handle clock out operation with error handling
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
      const result = await doClockOut();
      setLastOperation("Clock Out");
      setMessage("Successfully clocked out");
      setMessageType("success");
      console.log("Clock out result:", result);
    } catch (error: any) {
      const errorMessage = error.message || String(error);
      setMessage(`Clock out failed: ${errorMessage}`);
      setMessageType("error");
      console.error("Clock out error:", error);
    }
  }

  console.log({ ClockControls_refresh: refreshToken });
  // Show setup message if no refresh token
  if (!refreshToken) {
    return (
      <div className="clock-controls">
        <h2>Manual Clock Operations</h2>
        <p style={{ color: "orange" }}>
          Please set up your refresh token first to enable clock operations.
        </p>
      </div>
    );
  }

  return (
    <div className="clock-controls">
      <h2>Manual Clock Operations</h2>

      <div style={{ marginBottom: "15px" }}>
        <button
          onClick={handleClockIn}
          disabled={busy}
          style={{ marginRight: "10px", padding: "10px 20px" }}
        >
          {busy ? "Processing..." : "Clock In"}
        </button>

        <button
          onClick={handleClockOut}
          disabled={busy}
          style={{ padding: "10px 20px" }}
        >
          {busy ? "Processing..." : "Clock Out"}
        </button>
      </div>

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

      {accessToken && (
        <p style={{ fontSize: "0.9em", color: "gray", marginTop: "10px" }}>
          ✓ Access token available for API calls
        </p>
      )}
    </div>
  );
}

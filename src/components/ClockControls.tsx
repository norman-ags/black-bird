import { useState } from "react";
import { useClock } from "../hooks/use-clock";

/**
 * ClockControls component - provides manual clock in/out operations
 * Backend handles token management automatically
 */
export default function ClockControls() {
  const { busy, doClockIn, doClockOut } = useClock();
  const [lastOperation, setLastOperation] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<"success" | "error" | "info">(
    "info"
  );

  /**
   * Handle clock in operation with error handling
   */
  async function handleClockIn() {
    setMessage(null);
    try {
      const result = await doClockIn();
      setLastOperation("Clock In");
      if (result) {
        setMessage("Successfully clocked in");
        setMessageType("success");
      } else {
        setMessage("Clock in failed");
        setMessageType("error");
      }
      console.log("Clock in result:", result);
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      setMessage(`Clock in failed: ${errorMessage}`);
      setMessageType("error");
      console.error("Clock in error:", error);
    }
  }

  /**
   * Handle clock out operation with error handling
   */
  async function handleClockOut() {
    setMessage(null);
    try {
      const result = await doClockOut();
      setLastOperation("Clock Out");
      if (result) {
        setMessage("Successfully clocked out");
        setMessageType("success");
      } else {
        setMessage("Clock out failed");
        setMessageType("error");
      }
      console.log("Clock out result:", result);
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      setMessage(`Clock out failed: ${errorMessage}`);
      setMessageType("error");
      console.error("Clock out error:", error);
    }
  }

  // Note: Backend now handles all token management automatically

  return (
    <div className="clock-controls">
      <h2>Manual Clock Operations</h2>

      <div style={{ marginBottom: "15px" }}>
        <button
          type="button"
          onClick={handleClockIn}
          disabled={busy}
          style={{ marginRight: "10px", padding: "10px 20px" }}
        >
          {busy ? "Processing..." : "Clock In"}
        </button>

        <button
          type="button"
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
    </div>
  );
}

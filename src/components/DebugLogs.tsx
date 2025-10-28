import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LogEntry {
  id: string;
  timestamp: string;
  action: string;
  status: string;
  details: string;
  metadata: {
    duration?: number;
    trigger_type?: string;
    api_endpoint?: string;
    error_code?: string;
  };
}

export default function DebugLogs() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [debugInfo, setDebugInfo] = useState<any>(null);
  const [showDebugInfo, setShowDebugInfo] = useState(false);

  const fetchLogs = async () => {
    setLoading(true);
    try {
      const result = await invoke<LogEntry[]>("get_activity_logs", { limit: 50 });
      setLogs(result);
    } catch (error) {
      console.error("Failed to fetch logs:", error);
    } finally {
      setLoading(false);
    }
  };

  const fetchFailedLogs = async () => {
    setLoading(true);
    try {
      const result = await invoke<LogEntry[]>("get_filtered_activity_logs", {
        status_filter: "failed",
        limit: 20
      });
      setLogs(result);
    } catch (error) {
      console.error("Failed to fetch failed logs:", error);
    } finally {
      setLoading(false);
    }
  };

  const clearLogs = async () => {
    try {
      await invoke("clear_activity_logs");
      setLogs([]);
    } catch (error) {
      console.error("Failed to clear logs:", error);
    }
  };

  const fetchDebugInfo = async () => {
    try {
      const result = await invoke("debug_logging_status");
      setDebugInfo(result);
      setShowDebugInfo(true);
    } catch (error) {
      console.error("Failed to fetch debug info:", error);
      setDebugInfo({ error: String(error) });
      setShowDebugInfo(true);
    }
  };

  const reinitializeLogger = async () => {
    try {
      const result = await invoke<string>("reinitialize_logger");
      alert(`Logger reinitialization result: ${result}`);
      // Refresh logs after reinitialization
      fetchLogs();
    } catch (error) {
      console.error("Failed to reinitialize logger:", error);
      alert(`Failed to reinitialize logger: ${String(error)}`);
    }
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: we want to run this only once
  useEffect(() => {
    fetchLogs();
  }, []);

  return (
    <div style={{ padding: "20px", maxHeight: "80vh", overflow: "auto" }}>
      <h2>Debug Logs</h2>

      <div style={{ marginBottom: "20px" }}>
        <button
          onClick={fetchLogs}
          disabled={loading}
          type="button"
          style={{ marginRight: "10px", padding: "8px 16px" }}
        >
          {loading ? "Loading..." : "Refresh All Logs"}
        </button>
        <button
          onClick={fetchFailedLogs}
          type="button"
          disabled={loading}
          style={{ marginRight: "10px", padding: "8px 16px", backgroundColor: "#ef4444", color: "white", border: "none", borderRadius: "4px" }}
        >
          Show Failed Only
        </button>
        <button
          onClick={clearLogs}
          type="button"
          style={{ padding: "8px 16px", backgroundColor: "#6b7280", color: "white", border: "none", borderRadius: "4px", marginRight: "10px" }}
        >
          Clear All Logs
        </button>
        <button
          onClick={fetchDebugInfo}
          type="button"
          style={{ padding: "8px 16px", backgroundColor: "#3b82f6", color: "white", border: "none", borderRadius: "4px", marginRight: "10px" }}
        >
          Debug Info
        </button>
        <button
          onClick={reinitializeLogger}
          type="button"
          style={{ padding: "8px 16px", backgroundColor: "#f59e0b", color: "white", border: "none", borderRadius: "4px" }}
        >
          Fix Logger
        </button>
      </div>

      <div style={{ fontSize: "12px" }}>
        <strong>Total logs: {logs.length}</strong>
      </div>

      {showDebugInfo && debugInfo && (
        <div style={{
          marginTop: "20px",
          padding: "15px",
          backgroundColor: "#f8f9fa",
          border: "1px solid #dee2e6",
          borderRadius: "4px",
          fontSize: "12px"
        }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "10px" }}>
            <h3 style={{ margin: 0, fontSize: "14px" }}>Debug Information</h3>
            <button
              onClick={() => setShowDebugInfo(false)}
              type="button"
              style={{ padding: "4px 8px", fontSize: "10px" }}
            >
              Close
            </button>
          </div>
          <pre style={{
            fontSize: "11px",
            whiteSpace: "pre-wrap",
            wordBreak: "break-word",
            maxHeight: "300px",
            overflow: "auto",
            backgroundColor: "#ffffff",
            padding: "10px",
            border: "1px solid #e9ecef",
            borderRadius: "3px"
          }}>
            {JSON.stringify(debugInfo, null, 2)}
          </pre>
        </div>
      )}

      <div style={{ marginTop: "10px" }}>
        {logs.map((log) => (
          <div
            key={log.id}
            style={{
              border: "1px solid #e5e7eb",
              padding: "10px",
              marginBottom: "10px",
              borderRadius: "4px",
              backgroundColor: log.status === "failed" ? "#fef2f2" : log.status === "success" ? "#f0fdf4" : "#f9fafb"
            }}
          >
            <div style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginBottom: "5px"
            }}>
              <strong style={{
                color: log.status === "failed" ? "#dc2626" : log.status === "success" ? "#16a34a" : "#374151"
              }}>
                {log.action.toUpperCase()} - {log.status.toUpperCase()}
              </strong>
              <small style={{ color: "#6b7280" }}>
                {new Date(log.timestamp).toLocaleString()}
              </small>
            </div>

            <div style={{ marginBottom: "5px", fontSize: "14px" }}>
              {log.details}
            </div>

            {(log.metadata.error_code || log.metadata.trigger_type || log.metadata.duration) && (
              <div style={{ fontSize: "12px", color: "#6b7280" }}>
                {log.metadata.trigger_type && <span>Trigger: {log.metadata.trigger_type} | </span>}
                {log.metadata.duration && <span>Duration: {log.metadata.duration}ms | </span>}
                {log.metadata.error_code && <span style={{ color: "#dc2626" }}>Error: {log.metadata.error_code}</span>}
              </div>
            )}
          </div>
        ))}

        {logs.length === 0 && !loading && (
          <p style={{ textAlign: "center", color: "#6b7280", marginTop: "40px" }}>
            No logs found. Try performing some operations (manual clock-in, etc.) to generate logs.
          </p>
        )}
      </div>
    </div>
  );
}
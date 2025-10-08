import React, { useState } from "react";
import { ScheduleConfig } from "./ScheduleConfig";
import BackendSchedulerControls from "./BackendSchedulerControls";

/**
 * Unified Auto Schedule Component
 *
 * Combines schedule configuration with backend scheduler controls
 * into a single coherent interface, eliminating the need for separate tabs.
 */
export const AutoSchedule: React.FC = () => {
  const [activeSection, setActiveSection] = useState<"config" | "controls">(
    "config"
  );

  return (
    <div style={{ maxWidth: "800px", margin: "0 auto" }}>
      {/* Header */}
      <div style={{ textAlign: "center", marginBottom: "32px" }}>
        <h2>📅 Automatic Schedule Management</h2>
        <p>Configure your work schedule and control the automatic scheduler.</p>
      </div>

      {/* Section Toggle */}
      <div
        style={{
          display: "flex",
          gap: "8px",
          marginBottom: "24px",
          justifyContent: "center",
        }}
      >
        <button
          type="button"
          onClick={() => setActiveSection("config")}
          style={{
            padding: "8px 16px",
            backgroundColor: activeSection === "config" ? "#3b82f6" : "#f3f4f6",
            color: activeSection === "config" ? "white" : "#374151",
            border: "none",
            borderRadius: "6px",
            cursor: "pointer",
          }}
        >
          ⚙️ Schedule Configuration
        </button>
        <button
          type="button"
          onClick={() => setActiveSection("controls")}
          style={{
            padding: "8px 16px",
            backgroundColor:
              activeSection === "controls" ? "#3b82f6" : "#f3f4f6",
            color: activeSection === "controls" ? "white" : "#374151",
            border: "none",
            borderRadius: "6px",
            cursor: "pointer",
          }}
        >
          🎮 Scheduler Controls
        </button>
      </div>

      {/* Content Sections */}
      {activeSection === "config" && (
        <div>
          <ScheduleConfig />
        </div>
      )}

      {activeSection === "controls" && (
        <div>
          <BackendSchedulerControls />
        </div>
      )}
    </div>
  );
};

export default AutoSchedule;

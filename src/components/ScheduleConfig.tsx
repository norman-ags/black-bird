import React, { useState } from "react";
import type { WorkSchedule } from "../types/schedule";
import { useScheduleQuery } from "../queries/schedule";
import { useScheduleMutation } from "../mutations/schedule";

interface ScheduleConfigProps {
  schedule: WorkSchedule;
  onScheduleSubmit: (schedule: WorkSchedule) => Promise<void>;
  isSaving?: boolean;
}

interface NotificationState {
  type: "success" | "error" | null;
  message: string;
}

const TIMEZONE_OPTIONS = [
  { value: "America/New_York", label: "Eastern Time (EST/EDT)" },
  { value: "America/Chicago", label: "Central Time (CST/CDT)" },
  { value: "America/Denver", label: "Mountain Time (MST/MDT)" },
  { value: "America/Los_Angeles", label: "Pacific Time (PST/PDT)" },
  { value: "UTC", label: "UTC (Coordinated Universal Time)" },
];

const generateTimeOptions = () => {
  const options = [];
  for (let hour = 6; hour <= 12; hour++) {
    for (let minute = 0; minute < 60; minute += 15) {
      const time24 = `${hour.toString().padStart(2, "0")}:${minute
        .toString()
        .padStart(2, "0")}`;
      const hour12 = hour > 12 ? hour - 12 : hour === 0 ? 12 : hour;
      const ampm = hour >= 12 ? "PM" : "AM";
      const time12 = `${hour12}:${minute.toString().padStart(2, "0")} ${ampm}`;
      options.push({ value: time24, label: time12 });
    }
  }
  return options;
};

// Default schedule configuration
const DEFAULT_SCHEDULE: WorkSchedule = {
  autoScheduleEnabled: false,
  clockInTime: "09:00",
  timezone: "America/New_York",
  minWorkDurationMinutes: 540, // 9 hours
};

export const ScheduleConfig = () => {
  const { data: schedule, isLoading } = useScheduleQuery();
  const { mutateAsync: submitSchedule, isPending } = useScheduleMutation();

  const handleSubmit = async (schedule: WorkSchedule) => {
    await submitSchedule(schedule);
  };

  if (isLoading) {
    return <div>Loading your schedule...</div>;
  }

  return (
    <ScheduleConfigForm
      onScheduleSubmit={handleSubmit}
      schedule={schedule}
      isSaving={isPending}
    />
  );
};

const ScheduleConfigForm: React.FC<ScheduleConfigProps> = ({
  onScheduleSubmit,
  schedule,
  isSaving,
}) => {
  const [localSchedule, setLocalSchedule] = useState<WorkSchedule>(
    schedule ?? DEFAULT_SCHEDULE
  );
  const [notification, setNotification] = useState<NotificationState>({
    type: null,
    message: "",
  });
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);

  const handleChange = (field: keyof WorkSchedule, value: any) => {
    const updatedSchedule = { ...localSchedule, [field]: value };
    setLocalSchedule(updatedSchedule);
    setHasUnsavedChanges(true);
  };

  const showNotification = (type: "success" | "error", message: string) => {
    setNotification({ type, message });
    setTimeout(() => {
      setNotification({ type: null, message: "" });
    }, 3000);
  };
  const handleSave = async () => {
    try {
      // Save to secure storage
      await onScheduleSubmit(localSchedule);

      setHasUnsavedChanges(false);

      showNotification("success", "Schedule saved successfully!");
    } catch (error) {
      console.error("Failed to save schedule:", error);
      showNotification(
        "error",
        `Failed to save schedule: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }
  };

  const handleReset = () => {
    setHasUnsavedChanges(false);
    showNotification("success", "Changes reset successfully!");
  };

  const getExpectedClockOutTime = (): string => {
    if (!localSchedule.clockInTime) return "";
    const [hours, minutes] = localSchedule.clockInTime.split(":").map(Number);
    const clockInMinutes = hours * 60 + minutes;
    const clockOutMinutes =
      clockInMinutes + localSchedule.minWorkDurationMinutes;
    const clockOutHour = Math.floor(clockOutMinutes / 60) % 24;
    const clockOutMin = clockOutMinutes % 60;
    return `${clockOutHour.toString().padStart(2, "0")}:${clockOutMin
      .toString()
      .padStart(2, "0")}`;
  };

  const timeOptions = generateTimeOptions();
  const expectedClockOut = getExpectedClockOutTime();

  return (
    <div style={{ maxWidth: "600px", margin: "0 auto", padding: "24px" }}>
      <div style={{ textAlign: "center", marginBottom: "32px" }}>
        <h2>⏰ Automatic Schedule Configuration</h2>
        <p>
          Configure your preferred work schedule with automatic clock-in/out
          functionality.
        </p>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
        <div>
          <label style={{ display: "flex", alignItems: "center", gap: "12px" }}>
            <input
              type="checkbox"
              checked={localSchedule.autoScheduleEnabled}
              onChange={(e) =>
                handleChange("autoScheduleEnabled", e.target.checked)
              }
            />
            Enable Automatic Scheduling
          </label>
          <small style={{ color: "#6b7280", fontSize: "12px" }}>
            When enabled, the app will automatically clock you in and out based
            on your configured times.
          </small>
        </div>

        <div>
          <label htmlFor="clockInTime">Preferred Clock-in Time</label>
          <select
            id="clockInTime"
            value={localSchedule.clockInTime}
            onChange={(e) => handleChange("clockInTime", e.target.value)}
            disabled={!localSchedule.autoScheduleEnabled}
            style={{ width: "100%", padding: "12px", marginTop: "8px" }}
          >
            <option value="">Select time...</option>
            {timeOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label htmlFor="timezone">Timezone</label>
          <select
            id="timezone"
            value={localSchedule.timezone}
            onChange={(e) => handleChange("timezone", e.target.value)}
            disabled={!localSchedule.autoScheduleEnabled}
            style={{ width: "100%", padding: "12px", marginTop: "8px" }}
          >
            {TIMEZONE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label htmlFor="minWorkDuration">Minimum Work Duration (hours)</label>
          <select
            id="minWorkDuration"
            value={localSchedule.minWorkDurationMinutes / 60}
            onChange={(e) =>
              handleChange(
                "minWorkDurationMinutes",
                parseInt(e.target.value) * 60
              )
            }
            disabled={!localSchedule.autoScheduleEnabled}
            style={{ width: "100%", padding: "12px", marginTop: "8px" }}
          >
            <option value={8}>8 hours</option>
            <option value={9}>9 hours (recommended)</option>
            <option value={10}>10 hours</option>
          </select>
        </div>

        {localSchedule.autoScheduleEnabled && localSchedule.clockInTime && (
          <div
            style={{
              background: "#f0f9ff",
              padding: "16px",
              borderRadius: "8px",
            }}
          >
            <h3>📅 Schedule Preview</h3>
            <div>
              <div>Clock-in: {localSchedule.clockInTime}</div>
              <div>Earliest Clock-out: {expectedClockOut}</div>
              <div>
                Timezone:{" "}
                {
                  TIMEZONE_OPTIONS.find(
                    (tz) => tz.value === localSchedule.timezone
                  )?.label
                }
              </div>
            </div>
          </div>
        )}

        <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
          <button
            onClick={handleSave}
            disabled={!hasUnsavedChanges || isSaving}
            style={{ padding: "12px 24px" }}
          >
            {isSaving ? "💾 Saving..." : "💾 Save Configuration"}
          </button>

          {hasUnsavedChanges && (
            <button onClick={handleReset} style={{ padding: "12px 24px" }}>
              🔄 Reset Changes
            </button>
          )}
        </div>

        {/* Notification display */}
        {notification.type && (
          <div
            style={{
              background:
                notification.type === "success" ? "#d1fae5" : "#fecaca",
              color: notification.type === "success" ? "#065f46" : "#991b1b",
              padding: "12px",
              borderRadius: "8px",
              textAlign: "center",
              border: `1px solid ${
                notification.type === "success" ? "#a7f3d0" : "#fca5a5"
              }`,
            }}
          >
            {notification.type === "success" ? "✅" : "❌"}{" "}
            {notification.message}
          </div>
        )}

        {hasUnsavedChanges && !notification.type && (
          <div
            style={{
              background: "#fef3c7",
              padding: "12px",
              borderRadius: "8px",
              textAlign: "center",
            }}
          >
            ⚠️ You have unsaved changes. Don't forget to save your
            configuration!
          </div>
        )}
      </div>
    </div>
  );
};

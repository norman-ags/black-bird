import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

// Define AttendanceItem type based on the API response
interface AttendanceItem {
  work_date: string;
  attendance_status: string;
  date_time_in: string | null;
  date_time_out: string | null;
  is_restday: boolean | null;
}

export const ATTENDANCE_QUERY_KEY = "attendance-status";

/**
 * Fetch current attendance status from EMAPTA API
 */
const fetchAttendanceStatus = async (): Promise<AttendanceItem | null> => {
  return await invoke<AttendanceItem | null>("api_get_attendance_status");
};

/**
 * React Query hook for fetching current attendance status
 */
export const useAttendanceQuery = () => {
  return useQuery({
    queryKey: [ATTENDANCE_QUERY_KEY],
    queryFn: fetchAttendanceStatus,
    refetchInterval: 30000, // Refetch every 30 seconds
    retry: 3,
    staleTime: 15000, // Consider data stale after 15 seconds
  });
};

/**
 * Helper to determine if user is currently clocked in based on attendance data
 */
export const isCurrentlyClocked = (
  attendance: AttendanceItem | null
): boolean => {
  if (!attendance) return false;

  return (
    attendance.attendance_status === "Started" &&
    attendance.date_time_in != null &&
    attendance.date_time_out == null
  );
};

/**
 * Helper to determine if user has completed their shift for the day
 */
export const hasCompletedShift = (
  attendance: AttendanceItem | null
): boolean => {
  if (!attendance) return false;

  return (
    attendance.attendance_status === "Completed" &&
    attendance.date_time_in != null &&
    attendance.date_time_out != null
  );
};

/**
 * Helper to get clock in time from attendance data
 */
export const getClockInTime = (
  attendance: AttendanceItem | null
): string | null => {
  if (!attendance || !isCurrentlyClocked(attendance)) return null;
  return attendance.date_time_in;
};

/**
 * Helper to calculate expected clock out time (9 hours 10 minutes after clock in)
 */
export const getExpectedClockOutTime = (
  attendance: AttendanceItem | null
): string | null => {
  const clockInTime = getClockInTime(attendance);
  if (!clockInTime) return null;

  const clockInDate = new Date(clockInTime);
  const expectedClockOut = new Date(clockInDate.getTime() + 550 * 60 * 1000); // 9 hours 10 minutes
  return expectedClockOut.toISOString();
};

/**
 * Helper to get completed shift details
 */
export const getCompletedShiftDetails = (
  attendance: AttendanceItem | null
): { clockInTime: string; clockOutTime: string } | null => {
  if (!hasCompletedShift(attendance) || !attendance) return null;

  const clockInTime = attendance.date_time_in;
  const clockOutTime = attendance.date_time_out;

  if (!clockInTime || !clockOutTime) return null;

  return {
    clockInTime,
    clockOutTime
  };
};

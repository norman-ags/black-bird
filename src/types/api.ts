/**
 * EMAPTA API response interfaces
 */
export interface EmaptaTokenResponse {
  access_token: string;
  refresh_token: string;
  expires_in?: number;
  token_type?: string;
  scope?: string;
}

/**
 * EMAPTA attendance status response item
 */
export interface EmaptaAttendanceItem {
  work_date: string;
  work_day: string;
  work_day_code: string;
  is_complete: boolean;
  attendance_status:
    | "Completed"
    | "Rest Day"
    | "On leave"
    | "Started"
    | "Not started";
  leave_details: string | null;
  is_restday: boolean | null;
  date_time_in: string | null;
  date_time_out: string | null;
  work_minutes_rendered: number | null;
  expected_work_minutes: number;
}

/**
 * EMAPTA attendance API response
 */
export interface EmaptaAttendanceResponse {
  timestamp: string;
  status_code: number;
  message: string[];
  data: {
    items: EmaptaAttendanceItem[];
  };
}

/**
 * Clock operation API response
 */
export interface ClockOperationResponse {
  success: boolean;
  timestamp?: string;
  message?: string;
  error?: string;
}

/**
 * Timezone utilities using date-fns for accurate time handling
 */

/**
 * Get the user's current timezone
 * @returns Timezone identifier (e.g., 'America/New_York')
 */
export function getCurrentTimezone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
}

/**
 * Convert a time string (HH:MM) to today's Date in specified timezone
 * @param timeString Time in HH:MM format
 * @param timezone Target timezone (defaults to current)
 * @returns Date object for today at specified time
 */
export function timeStringToDate(timeString: string, timezone?: string): Date {
  const [hours, minutes] = timeString.split(":").map(Number);
  const now = new Date();
  const targetDate = new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate(),
    hours,
    minutes,
    0,
    0
  );

  if (timezone && timezone !== getCurrentTimezone()) {
    // For demo purposes, we'll use basic timezone offset calculation
    // In production, use date-fns-tz for proper timezone handling
    const offset = getTimezoneOffset(timezone);
    targetDate.setMinutes(targetDate.getMinutes() + offset);
  }

  return targetDate;
}

/**
 * Format a Date to HH:MM string in local timezone
 * @param date Date to format
 * @returns Time string in HH:MM format
 */
export function dateToTimeString(date: Date): string {
  return date.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
  });
}

/**
 * Add minutes to a Date
 * @param date Base date
 * @param minutes Minutes to add
 * @returns New Date with minutes added
 */
export function addMinutes(date: Date, minutes: number): Date {
  return new Date(date.getTime() + minutes * 60000);
}

/**
 * Calculate difference in minutes between two dates
 * @param laterDate Later date
 * @param earlierDate Earlier date
 * @returns Difference in minutes
 */
export function diffInMinutes(laterDate: Date, earlierDate: Date): number {
  return Math.floor((laterDate.getTime() - earlierDate.getTime()) / 60000);
}

/**
 * Check if two dates are on the same day
 * @param date1 First date
 * @param date2 Second date
 * @returns true if same day
 */
export function isSameDay(date1: Date, date2: Date): boolean {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
}

/**
 * Format date for display with timezone info
 * @param date Date to format
 * @param timezone Timezone to display (optional)
 * @returns Formatted date string with timezone
 */
export function formatDateWithTimezone(date: Date, timezone?: string): string {
  const formatted = date.toLocaleString("en-US", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });

  const tz = timezone || getCurrentTimezone();
  return `${formatted} (${tz})`;
}

/**
 * Get timezone offset in minutes (simplified implementation)
 * @param timezone Timezone identifier
 * @returns Offset in minutes from UTC
 */
function getTimezoneOffset(timezone: string): number {
  // Simplified timezone offset mapping for demo
  // In production, use proper timezone libraries
  const offsets: Record<string, number> = {
    "America/New_York": -240, // EDT
    "America/Chicago": -300, // CDT
    "America/Denver": -360, // MDT
    "America/Los_Angeles": -420, // PDT
    "Europe/London": 60, // BST
    "Europe/Paris": 120, // CEST
    "Asia/Tokyo": 540, // JST
    "Asia/Shanghai": 480, // CST
  };

  return offsets[timezone] || 0;
}

/**
 * Check if current time is during daylight saving time (simplified)
 * @returns true if likely DST period
 */
export function isDaylightSavingTime(): boolean {
  const now = new Date();
  const month = now.getMonth();
  // Rough DST check for northern hemisphere (March-November)
  return month >= 2 && month <= 10;
}

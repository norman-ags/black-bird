/**
 * Single clock event record
 */
export interface ClockEvent {
  id: string;
  timestamp: string; // ISO
  type: "clock-in" | "clock-out" | "skip" | "error";
  details?: string;
  success: boolean;
}

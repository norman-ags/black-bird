import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useClock() {
  const [busy, setBusy] = useState(false);

  async function doClockIn() {
    setBusy(true);
    try {
      // Call backend API command instead of frontend service
      console.log("[useClock] Attempting manual clock-in...");
      const success = await invoke<boolean>("api_manual_clock_in");
      console.log("[useClock] Clock-in result:", success);
      return success;
    } catch (error) {
      console.error("[useClock] Clock-in error:", error);
      throw error; // Re-throw to let the UI handle it
    } finally {
      setBusy(false);
    }
  }

  async function doClockOut() {
    setBusy(true);
    try {
      // Call backend API command instead of frontend service
      console.log("[useClock] Attempting manual clock-out...");
      const success = await invoke<boolean>("api_manual_clock_out");
      console.log("[useClock] Clock-out result:", success);
      return success;
    } catch (error) {
      console.error("[useClock] Clock-out error:", error);
      throw error; // Re-throw to let the UI handle it
    } finally {
      setBusy(false);
    }
  }

  return { busy, doClockIn, doClockOut };
}

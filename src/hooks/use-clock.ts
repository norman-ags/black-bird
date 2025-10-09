import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useClock() {
  const [busy, setBusy] = useState(false);

  async function doClockIn() {
    setBusy(true);
    try {
      // Call backend API command instead of frontend service
      const success = await invoke<boolean>("api_manual_clock_in");
      return success;
    } finally {
      setBusy(false);
    }
  }

  async function doClockOut() {
    setBusy(true);
    try {
      // Call backend API command instead of frontend service
      const success = await invoke<boolean>("api_manual_clock_out");
      return success;
    } finally {
      setBusy(false);
    }
  }

  return { busy, doClockIn, doClockOut };
}

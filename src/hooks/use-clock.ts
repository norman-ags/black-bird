import { useState } from "react";
import { clockIn, clockOut } from "../services/clock-service";

export function useClock(accessToken: string | null) {
  const [busy, setBusy] = useState(false);

  async function doClockIn() {
    setBusy(true);
    try {
      const res = await clockIn(accessToken);
      return res;
    } finally {
      setBusy(false);
    }
  }

  async function doClockOut() {
    setBusy(true);
    try {
      const res = await clockOut(accessToken);
      return res;
    } finally {
      setBusy(false);
    }
  }

  return { busy, doClockIn, doClockOut };
}

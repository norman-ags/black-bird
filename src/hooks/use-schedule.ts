import { useBackendSchedule } from "./use-backend-schedule";

/**
 * @deprecated This hook is deprecated. Use useBackendSchedule() instead.
 *
 * This is a compatibility wrapper for the old useSchedule hook.
 * All new code should use useBackendSchedule() directly.
 */
export const useSchedule = () => {
  // Just redirect to the backend scheduler
  return useBackendSchedule();
};

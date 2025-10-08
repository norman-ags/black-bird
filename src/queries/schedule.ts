import { useQuery } from "@tanstack/react-query";
import { loadSchedule } from "../services/storage-service";

export const SCHEDULE_QUERY_KEY = "clock-in-out-schedule";
export const useScheduleQuery = () => {
  return useQuery({
    queryFn: loadSchedule,
    queryKey: [SCHEDULE_QUERY_KEY],
  });
};

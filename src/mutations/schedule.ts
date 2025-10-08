import { useMutation } from "@tanstack/react-query";
import { saveSchedule } from "../services/storage-service";
import { WorkSchedule } from "../types/schedule";

export const useScheduleMutation = () => {
  return useMutation({
    mutationFn: async (newSchedule: WorkSchedule, context) => {
      // Save the new schedule to storage
      await saveSchedule(newSchedule);
      context.client.invalidateQueries({ queryKey: ["clock-in-out-schedule"] });
      return newSchedule;
    },
    onMutate: (newSchedule, context) => {
      context.client.setQueryData(["clock-in-out-schedule"], newSchedule);
    },
  });
};

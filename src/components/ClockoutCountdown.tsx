import React from "react";

interface ClockoutCountdownProps {
  isCurrentlyClockedIn: boolean;
  expectedClockOutTime: string | null;
}

export function ClockoutCountdown({
  isCurrentlyClockedIn,
  expectedClockOutTime,
}: ClockoutCountdownProps) {
  const [timeRemaining, setTimeRemaining] = React.useState<string | null>(null);
  const getTimeRemaining = () => {
    if (!isCurrentlyClockedIn || !expectedClockOutTime) return null;

    const now = new Date();
    const clockOutTime = new Date(expectedClockOutTime);
    const diffMs = clockOutTime.getTime() - now.getTime();

    if (diffMs <= 0) return "Ready to clock out";

    const hours = Math.floor(diffMs / (1000 * 60 * 60));
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    return `${hours}h ${minutes}m remaining`;
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: we only want to run this once
  React.useEffect(() => {
    const updateCountdown = () => {
      const remaining = getTimeRemaining();
      setTimeRemaining(remaining);
    };

    updateCountdown(); // Initial call

    const intervalId = setInterval(updateCountdown, 1000); // Update every second

    return () => clearInterval(intervalId); // Cleanup on unmount
  }, []);

  return (
    <div
      style={{
        fontSize: "16px",
        color: "#3b82f6",
        fontWeight: "500",
      }}
    >
      {timeRemaining}
    </div>
  );
}

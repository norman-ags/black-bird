import { useState } from "react";
import "./App.css";
import TokenSetup from "./components/TokenSetup";
import StatusScreen from "./components/StatusScreen";
import ClockControls from "./components/ClockControls";
import { AuthProvider } from "./provider/AuthProvider";
import { QueryClientProvider, QueryClient } from "@tanstack/react-query";
import { useAuth } from "./hooks/use-auth";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
    },
  },
});

const AppContent = () => {
  const { accessToken } = useAuth();
  const [showEmergencyOverride, setShowEmergencyOverride] = useState(false);

  // If not authenticated, show setup
  if (!accessToken) {
    return (
      <main className="container">
        <h1>Black Bird — Clock Automation</h1>
        <TokenSetup />
      </main>
    );
  }

  // Ultra-simplified interface
  return (
    <main className="container">
      <h1>Black Bird — Clock Automation</h1>

      <StatusScreen />

      {/* Emergency manual override section */}
      <div
        style={{
          marginTop: "32px",
          borderTop: "1px solid #e5e7eb",
          paddingTop: "16px",
        }}
      >
        <button
          type="button"
          onClick={() => setShowEmergencyOverride(!showEmergencyOverride)}
          style={{
            padding: "8px 16px",
            fontSize: "13px",
            backgroundColor: showEmergencyOverride ? "#ef4444" : "#f59e0b",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: "pointer",
            marginBottom: "16px",
          }}
        >
          {showEmergencyOverride
            ? "✕ Hide Emergency Controls"
            : "⚠️ Emergency Manual Override"}
        </button>

        {showEmergencyOverride && (
          <div
            style={{
              padding: "16px",
              backgroundColor: "#fef3c7",
              border: "1px solid #f59e0b",
              borderRadius: "6px",
            }}
          >
            <p
              style={{
                margin: "0 0 16px 0",
                fontSize: "14px",
                color: "#92400e",
                fontWeight: "500",
              }}
            >
              ⚠️ Emergency use only. The app will return to automatic mode after
              manual operations.
            </p>
            <ClockControls />
          </div>
        )}
      </div>
    </main>
  );
};

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <AppContent />
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;

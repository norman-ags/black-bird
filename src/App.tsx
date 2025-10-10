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
      staleTime: 1000 * 60 * 1, // 1 minute
    },
  },
});

const AppContent = () => {
  const { refreshToken, loading } = useAuth();
  const [showEmergencyOverride, setShowEmergencyOverride] = useState(false);
  const [activeScreen, setActiveScreen] = useState<"status" | "setup" | null>(
    "status"
  );

  // Show loading while checking for stored tokens
  if (loading) {
    return (
      <main className="container">
        <h1>Black Bird</h1>
        <div style={{ textAlign: "center", padding: "48px 24px" }}>
          <p>Loading...</p>
        </div>
      </main>
    );
  }

  console.log({ refreshToken });

  // If no refresh token stored, show setup (first time user)
  if (!refreshToken || activeScreen === "setup") {
    return (
      <main className="container">
        <h1>Black Bird</h1>
        <TokenSetup onSave={() => setActiveScreen("status")} />
      </main>
    );
  }

  // If refresh token exists but no access token, we'll get one automatically
  // Show status screen regardless - the backend will handle token refresh

  // Ultra-simplified interface
  return (
    <main className="container">
      <h1>BlackBird</h1>

      <StatusScreen />

      <button
        type="button"
        onClick={() => setActiveScreen("setup")}
        style={{
          padding: "8px 16px",
          fontSize: "13px",
          backgroundColor: "#f59e0b",
          color: "white",
          border: "none",
          borderRadius: "4px",
          cursor: "pointer",
          marginBottom: "16px",
        }}
      >
        Go to setup
      </button>
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

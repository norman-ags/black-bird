import { useState, useId } from "react";
import { invoke } from '@tauri-apps/api/core';
import { useAuth } from "../hooks/use-auth";

/**
 * TokenSetup component - collects refresh and access tokens from user
 * Supports both tokens as per automatic attendance tracking scenario
 */
export default function TokenSetup({ onSave }: { onSave?: () => void }) {
  const { loading, refreshToken, reloadTokens } = useAuth();
  const refreshTokenId = useId();
  const accessTokenId = useId();
  const [refreshTokenValue, setRefreshTokenValue] = useState("");
  const [accessTokenValue, setAccessTokenValue] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<"success" | "error" | "info">(
    "info"
  );

  console.log({ refreshToken });

  /**
   * Handle form submission for dual token setup and validation
   * @param e Form submit event
   */
  async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setMessage(null);

    // Validate both tokens are provided
    if (!refreshTokenValue.trim()) {
      setMessage("Please enter a refresh token");
      setMessageType("error");
      return;
    }

    if (!accessTokenValue.trim()) {
      setMessage("Please enter an access token");
      setMessageType("error");
      return;
    }

    try {
      // Use the shared token manager for initial setup
      await authenticateWithBothTokens(refreshTokenValue.trim(), accessTokenValue.trim());
      setMessage("Tokens saved and validated successfully! Setup complete.");
      setMessageType("success");

      // Clear inputs after successful save
      setRefreshTokenValue("");
      setAccessTokenValue("");

      // Reload tokens from storage to update AuthProvider state
      await reloadTokens();

      onSave?.();
    } catch (err: any) {
      const errorMessage = err.message || String(err);
      setMessage(`Setup failed: ${errorMessage}`);
      setMessageType("error");

      // Log detailed error for debugging
      console.error("Token setup failed:", err);
    }
  }

  /**
   * Authenticate with both refresh and access tokens using Tauri command
   */
  async function authenticateWithBothTokens(refreshToken: string, accessToken: string) {
    // Use the new dual token setup command
    return await invoke('api_setup_dual_tokens', {
      refreshToken,
      accessToken
    });
  }

  /**
   * Handle input changes with basic sanitization
   */
  function handleRefreshTokenChange(e: React.ChangeEvent<HTMLInputElement>) {
    const newValue = e.target.value.trim();
    setRefreshTokenValue(newValue);

    // Clear previous messages when user starts typing
    if (message) {
      setMessage(null);
    }
  }

  function handleAccessTokenChange(e: React.ChangeEvent<HTMLInputElement>) {
    const newValue = e.target.value.trim();
    setAccessTokenValue(newValue);

    // Clear previous messages when user starts typing
    if (message) {
      setMessage(null);
    }
  }

  const isFormValid = refreshTokenValue.trim() && accessTokenValue.trim();

  console.log({ tokenSETUP: refreshToken });
  return (
    <div className="token-setup">
      <h2>Token Setup</h2>
      <p>Enter both EMAPTA tokens to enable automatic clock automation:</p>

      <form onSubmit={onSubmit}>
        <div style={{ marginBottom: "16px" }}>
          <label
            htmlFor={refreshTokenId}
            style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}
          >
            Refresh Token:
          </label>
          <input
            id={refreshTokenId}
            type="password"
            placeholder="Enter refresh token"
            value={refreshTokenValue}
            onChange={handleRefreshTokenChange}
            disabled={loading}
            style={{
              width: "400px",
              padding: "8px",
              fontSize: "14px",
              marginBottom: "8px"
            }}
          />
        </div>

        <div style={{ marginBottom: "16px" }}>
          <label
            htmlFor={accessTokenId}
            style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}
          >
            Access Token:
          </label>
          <input
            id={accessTokenId}
            type="password"
            placeholder="Enter access token"
            value={accessTokenValue}
            onChange={handleAccessTokenChange}
            disabled={loading}
            style={{
              width: "400px",
              padding: "8px",
              fontSize: "14px",
              marginBottom: "8px"
            }}
          />
        </div>

        <button
          type="submit"
          disabled={loading || !isFormValid}
          style={{
            padding: "10px 20px",
            fontSize: "16px",
            backgroundColor: isFormValid ? "#007bff" : "#ccc",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: isFormValid ? "pointer" : "not-allowed"
          }}
        >
          {loading ? "Setting up tokens..." : "Save Tokens & Complete Setup"}
        </button>
      </form>

      {refreshToken && (
        <p style={{ color: "green", marginTop: "16px" }}>
          ✓ Tokens stored and ready for automatic operation
        </p>
      )}

      {message && (
        <p
          style={{
            color:
              messageType === "success"
                ? "green"
                : messageType === "error"
                ? "red"
                : "blue",
            marginTop: "16px",
            padding: "8px",
            backgroundColor:
              messageType === "success"
                ? "#d4edda"
                : messageType === "error"
                ? "#f8d7da"
                : "#d1ecf1",
            border: `1px solid ${
              messageType === "success"
                ? "#c3e6cb"
                : messageType === "error"
                ? "#f5c6cb"
                : "#bee5eb"
            }`,
            borderRadius: "4px",
          }}
        >
          {message}
        </p>
      )}
    </div>
  );
}

import { useState } from "react";
import { isValidRefreshTokenFormat } from "../utils/validation";
import { useAuth } from "../hooks/use-auth";

/**
 * TokenSetup component - collects refresh token from user and validates it
 * Provides secure storage and validation of EMAPTA refresh tokens
 */
export default function TokenSetup() {
  const { authenticate, loading, refreshToken } = useAuth();
  const [value, setValue] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<"success" | "error" | "info">(
    "info"
  );

  console.log({ refreshToken });

  /**
   * Handle form submission for token validation and storage
   * @param e Form submit event
   */
  async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setMessage(null);

    // Client-side validation
    if (!value.trim()) {
      setMessage("Please enter a refresh token");
      setMessageType("error");
      return;
    }

    if (!isValidRefreshTokenFormat(value.trim())) {
      setMessage("Token format appears invalid. Please check your token.");
      setMessageType("error");
      return;
    }

    try {
      await authenticate(value.trim());
      setMessage("Token saved and validated successfully");
      setMessageType("success");
      setValue(""); // Clear input after successful save
    } catch (err: any) {
      const errorMessage = err.message || String(err);
      setMessage(`Error: ${errorMessage}`);
      setMessageType("error");

      // Log detailed error for debugging
      console.error("Token validation failed:", err);
    }
  }

  /**
   * Handle input change with basic sanitization
   * @param e Input change event
   */
  function handleInputChange(e: React.ChangeEvent<HTMLInputElement>) {
    const newValue = e.target.value.trim();
    setValue(newValue);

    // Clear previous messages when user starts typing
    if (message) {
      setMessage(null);
    }
  }

  console.log({ tokenSETUP: refreshToken });
  return (
    <div className="token-setup">
      <h2>Refresh Token Setup</h2>
      <p>Enter your EMAPTA refresh token to enable clock automation:</p>

      <form onSubmit={onSubmit}>
        <div>
          <input
            type="password"
            placeholder="Enter refresh token"
            value={value}
            onChange={handleInputChange}
            disabled={loading}
            style={{ width: "300px", marginRight: "10px" }}
          />
          <button type="submit" disabled={loading || !value.trim()}>
            {loading ? "Validating..." : "Save Token"}
          </button>
        </div>
      </form>

      {refreshToken && (
        <p style={{ color: "green" }}>✓ Token stored and ready for use</p>
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
            marginTop: "10px",
          }}
        >
          {message}
        </p>
      )}
    </div>
  );
}

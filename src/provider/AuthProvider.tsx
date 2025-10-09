import React, { useEffect, useState } from "react";
import {
  getStoredRefreshToken,
  validateAndStoreRefreshToken,
} from "../services/auth-service";

export const AuthContext = React.createContext<{
  refreshToken: string | null;
  accessToken: string | null;
  authenticate: (token: string) => Promise<void>;
  loading: boolean;
}>({
  authenticate: async () => {},
  loading: false,
  refreshToken: null,
  accessToken: null,
});

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [refreshToken, setRefreshToken] = useState<string | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(true); // Start with loading true

  useEffect(() => {
    (async () => {
      try {
        const stored = await getStoredRefreshToken();
        if (stored) {
          setRefreshToken(stored);
          // We could also try to get a fresh access token here
          // but we'll let the backend handle that automatically
        }
      } catch (error) {
        console.error("Failed to load stored refresh token:", error);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  async function authenticate(userToken: string) {
    setLoading(true);
    try {
      const tokenResponse = await validateAndStoreRefreshToken(userToken);
      setRefreshToken(tokenResponse.refresh_token);
      setAccessToken(tokenResponse.access_token);
    } catch (error) {
      console.error("Authentication failed:", error);
      throw error; // Re-throw so the UI can handle it
    } finally {
      setLoading(false);
    }
  }

  return (
    <AuthContext.Provider
      value={{ refreshToken, accessToken, authenticate, loading }}
    >
      {children}
    </AuthContext.Provider>
  );
};

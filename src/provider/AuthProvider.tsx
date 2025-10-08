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
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    (async () => {
      const stored = await getStoredRefreshToken();

      if (stored) setRefreshToken(stored);
    })();
  }, []);

  async function authenticate(userToken: string) {
    setLoading(true);
    try {
      const resp = await validateAndStoreRefreshToken(userToken);
      setRefreshToken(resp.refresh_token);
      setAccessToken(resp.access_token);
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

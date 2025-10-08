/**
 * Auth token response from EMAPTA token exchange
 */
export interface TokenResponse {
  access_token: string;
  refresh_token: string;
  expires_in?: number;
  token_type?: string;
}

/**
 * Stored refresh token metadata
 */
export interface RefreshTokenStore {
  encrypted: string;
  expiresAt?: string | null;
}

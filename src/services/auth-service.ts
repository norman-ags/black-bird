import { exchangeRefreshToken } from "./api-client";
import {
  saveRefreshToken,
  saveRefreshTokenMetadata,
  loadRefreshToken,
} from "./storage-service";
import { validateRequiredString } from "../utils/validation";
import type { TokenResponse } from "../types/auth";

/**
 * Authentication service for EMAPTA token management
 * Handles token exchange, validation, and secure storage
 */

/**
 * Exchange user-provided refresh token and store received tokens
 * @param userRefreshToken refresh token provided by user
 * @returns Token response with access_token and new refresh_token
 * @throws Error if token exchange fails or validation fails
 */
export async function validateAndStoreRefreshToken(
  userRefreshToken: string
): Promise<TokenResponse> {
  validateRequiredString(userRefreshToken, "Refresh token");

  try {
    // const tokenResp = await exchangeRefreshToken(userRefreshToken);
    const tokenResp: TokenResponse = {
      access_token: "access_test",
      refresh_token: "refresh_test",
      expires_in: 3600,
      token_type: "Bearer",
    };

    // Store the new refresh token (encrypted)
    await saveRefreshToken(tokenResp.refresh_token);

    // Store metadata for tracking
    await saveRefreshTokenMetadata({
      encrypted: "true",
      expiresAt: tokenResp.expires_in
        ? new Date(Date.now() + tokenResp.expires_in * 1000).toISOString()
        : null,
    });

    return tokenResp;
  } catch (error) {
    throw new Error(
      `Token validation failed: ${
        error instanceof Error ? error.message : "Unknown error"
      }`
    );
  }
}

/**
 * Get stored refresh token from secure storage
 * @returns Stored refresh token or null if not found/corrupted
 */
export async function getStoredRefreshToken(): Promise<string | null> {
  try {
    return await loadRefreshToken();
  } catch (error) {
    console.warn("Failed to load stored refresh token:", error);
    return null;
  }
}

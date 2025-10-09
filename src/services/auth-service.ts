import { invoke } from "@tauri-apps/api/core";
import { saveRefreshTokenMetadata, loadRefreshToken } from "./storage-service";
import { validateRequiredString } from "../utils/validation";
import type { TokenResponse } from "../types/auth";

/**
 * Authentication service for EMAPTA token management
 * All API calls are now handled by the backend for security
 */

/**
 * Exchange user-provided refresh token and store received tokens (via backend)
 * @param userRefreshToken refresh token provided by user
 * @returns Token response with access_token and new refresh_token
 * @throws Error if token exchange fails or validation fails
 */
export async function validateAndStoreRefreshToken(
  userRefreshToken: string
): Promise<TokenResponse> {
  validateRequiredString(userRefreshToken, "Refresh token");

  try {
    // Call backend API command instead of direct API call
    const tokenResponse = await invoke<TokenResponse>(
      "api_exchange_refresh_token",
      {
        refreshToken: userRefreshToken,
      }
    );

    // The backend already stores the tokens securely, but we keep this for compatibility
    await saveRefreshTokenMetadata({
      encrypted: "true",
      expiresAt: tokenResponse.expires_in
        ? new Date(Date.now() + tokenResponse.expires_in * 1000).toISOString()
        : null,
    });

    return tokenResponse;
  } catch (error) {
    console.error("Token validation error details:", error);

    let errorMessage = "Unknown error";
    if (error instanceof Error) {
      errorMessage = error.message;
    } else if (typeof error === "string") {
      errorMessage = error;
    } else if (error && typeof error === "object") {
      errorMessage = JSON.stringify(error);
    }

    throw new Error(`Token validation failed: ${errorMessage}`);
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

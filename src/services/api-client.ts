import { EMAPTA_TOKEN_ENDPOINT } from "../constants/api-endpoints";
import type { TokenResponse } from "../types/auth";

/**
 * Basic fetch wrapper with simple retry.
 * @param input fetch input
 * @param init fetch init
 */
/**
 * API client service with retry logic for EMAPTA API calls
 * Provides resilient HTTP requests with exponential backoff
 */

/**
 * Fetch with automatic retry logic and exponential backoff
 * @param url Request URL
 * @param options Fetch request options
 * @param maxRetries Maximum number of retry attempts (default: 3)
 * @returns Fetch Response object
 * @throws Error if all retry attempts fail
 */
export async function fetchWithRetry(
  url: string,
  options: RequestInit,
  maxRetries = 3
): Promise<Response> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch(url, options);

      // Return successful responses or non-retryable errors
      if (response.ok || !isRetryableStatus(response.status)) {
        return response;
      }

      // If not the last attempt and status is retryable, continue to retry
      if (attempt < maxRetries) {
        await delay(getBackoffDelay(attempt));
        continue;
      }

      // Last attempt failed, return the response for error handling
      return response;
    } catch (error) {
      console.warn(`API request attempt ${attempt} failed:`, error);

      if (attempt === maxRetries) {
        throw new Error(
          `API request failed after ${maxRetries} attempts: ${error}`
        );
      }

      await delay(getBackoffDelay(attempt));
    }
  }

  throw new Error("Unexpected error in fetchWithRetry");
}

/**
 * Check if HTTP status code is retryable
 * @param status HTTP status code
 * @returns true if status indicates a retryable error
 */
function isRetryableStatus(status: number): boolean {
  return status >= 500 || status === 429; // Server errors and rate limiting
}

/**
 * Calculate exponential backoff delay
 * @param attempt Current attempt number (1-based)
 * @returns Delay in milliseconds
 */
function getBackoffDelay(attempt: number): number {
  return Math.min(1000 * Math.pow(2, attempt - 1), 10000); // Max 10 seconds
}

/**
 * Utility function for async delays
 * @param ms Delay duration in milliseconds
 * @returns Promise that resolves after the delay
 */
function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Exchange refresh token for access token using EMAPTA endpoint
 * @param refreshToken user provided refresh token
 */
export async function exchangeRefreshToken(
  refreshToken: string
): Promise<TokenResponse> {
  const body = JSON.stringify({
    grant_type: "refresh_token",
    client_id: "EMAPTA-MYEMAPTAWEB",
    refresh_token: refreshToken,
    scope: "openid",
  });

  const res = await fetchWithRetry(EMAPTA_TOKEN_ENDPOINT, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Token exchange failed: ${res.status} ${text}`);
  }
  const data = await res.json();
  return data as TokenResponse;
}

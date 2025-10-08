import {
  EMAPTA_LOGIN_ENDPOINT,
  EMAPTA_LOGOUT_ENDPOINT,
} from "../constants/api-endpoints";
import { fetchWithRetry } from "./api-client";

/**
 * Clock service for manual EMAPTA clock in/out operations
 * Handles API calls with proper error handling and retry logic
 */

/**
 * Make authenticated POST request to EMAPTA endpoint
 * @param url API endpoint URL
 * @param accessToken Bearer token for authentication
 * @returns Parsed JSON response
 * @throws Error if request fails after retries
 */
async function postWithAuth(url: string, accessToken: string | null) {
  if (!accessToken) {
    throw new Error("Access token is required for clock operations");
  }

  const response = await fetchWithRetry(url, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      Authorization: `Bearer ${accessToken}`,
    },
    body: JSON.stringify({}),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `Clock operation failed: ${response.status} ${response.statusText} - ${errorText}`
    );
  }

  return response.json();
}

/**
 * Manual clock in operation
 * @param accessToken Valid access token for API authentication
 * @returns API response data
 * @throws Error if clock in fails
 */
export async function clockIn(accessToken: string | null) {
  try {
    const result = await postWithAuth(EMAPTA_LOGIN_ENDPOINT, accessToken);
    console.log("Clock in successful:", new Date().toISOString());
    return result;
  } catch (error) {
    console.error("Clock in failed:", error);
    throw error;
  }
}

/**
 * Manual clock out operation
 * @param accessToken Valid access token for API authentication
 * @returns API response data
 * @throws Error if clock out fails
 */
export async function clockOut(accessToken: string | null) {
  try {
    const result = await postWithAuth(EMAPTA_LOGOUT_ENDPOINT, accessToken);
    console.log("Clock out successful:", new Date().toISOString());
    return result;
  } catch (error) {
    console.error("Clock out failed:", error);
    throw error;
  }
}

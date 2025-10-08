import type { RefreshTokenStore } from "../types/auth";
import { encryptString, decryptString } from "../utils/crypto";
import { invoke } from "@tauri-apps/api/core";

/**
 * Secure storage service for refresh token management
 *
 * Provides encrypted secure storage for sensitive authentication data using
 * Tauri's secure filesystem APIs with automatic fallback to Web Crypto API + localStorage
 * for development environments.
 *
 * Features:
 * - Tauri secure filesystem storage (production)
 * - AES-GCM encryption for stored tokens
 * - Automatic fallback to localStorage (development)
 * - Metadata storage for token validation status
 * - Secure token lifecycle management
 */

/** Storage key for encrypted refresh token */
const STORAGE_KEY = "bb_refresh_token";

/**
 * Check if running in Tauri environment
 * @returns True if Tauri APIs are available
 */
const isTauriEnvironment = (): boolean => {
  return typeof window !== "undefined" && "__TAURI__" in window;
};

/**
 * Store data using Tauri secure storage
 * @param key Storage key
 * @param encryptedData Encrypted data to store
 */
async function storeTauriData(
  key: string,
  encryptedData: string
): Promise<void> {
  try {
    await invoke("store_encrypted_data", { key, encryptedData });
  } catch (error) {
    console.error("Tauri storage failed:", error);
    throw new Error(
      `Tauri storage failed: ${
        error instanceof Error ? error.message : "Unknown error"
      }`
    );
  }
}

/**
 * Retrieve data using Tauri secure storage
 * @param key Storage key
 * @returns Encrypted data or null if not found
 */
async function retrieveTauriData(key: string): Promise<string | null> {
  try {
    const result = await invoke<string | null>("retrieve_encrypted_data", {
      key,
    });

    console.log({ result });
    return result;
  } catch (error) {
    console.error("Tauri retrieval failed:", error);
    return null;
  }
}

/**
 * Delete data using Tauri secure storage
 * @param key Storage key
 */
async function deleteTauriData(key: string): Promise<void> {
  try {
    await invoke("delete_encrypted_data", { key });
  } catch (error) {
    console.error("Tauri deletion failed:", error);
    // Don't throw - deletion failures are not critical
  }
}

/**
 * Save refresh token with AES-GCM encryption
 *
 * Encrypts the provided refresh token using Web Crypto API and stores it
 * using Tauri secure storage (if available) or localStorage as fallback.
 *
 * @param token Refresh token string to store securely
 * @throws Error if encryption or storage operation fails
 */
export async function saveRefreshToken(token: string): Promise<void> {
  try {
    const encrypted = await encryptString(token);

    if (isTauriEnvironment()) {
      // Use Tauri secure storage
      await storeTauriData(STORAGE_KEY, encrypted);
      console.log("Refresh token saved securely via Tauri");
    } else {
      // Fallback to localStorage for development
      localStorage.setItem(STORAGE_KEY, encrypted);
      console.log("Refresh token saved via localStorage (development mode)");
    }
  } catch (error) {
    console.error("Failed to save refresh token:", error);
    throw new Error(
      `Failed to save refresh token: ${
        error instanceof Error ? error.message : "Unknown error"
      }`
    );
  }
}

/**
 * Load and decrypt stored refresh token
 *
 * Retrieves the encrypted token from Tauri secure storage (if available) or
 * localStorage and decrypts it using Web Crypto API. Returns null if no token
 * exists or decryption fails.
 *
 * @returns Decrypted refresh token string or null if not found/invalid
 */
export async function loadRefreshToken(): Promise<string | null> {
  try {
    let encrypted: string | null = null;

    console.log({ isTauriEnvironment: isTauriEnvironment() });
    if (isTauriEnvironment()) {
      // Use Tauri secure storage
      encrypted = await retrieveTauriData(STORAGE_KEY);
    } else {
      // Fallback to localStorage for development
      encrypted = localStorage.getItem(STORAGE_KEY);
    }

    if (!encrypted) {
      return null;
    }

    const decrypted = await decryptString(encrypted);
    return decrypted || null;
  } catch (error) {
    console.warn(
      "Failed to decrypt refresh token, clearing corrupted data:",
      error
    );
    // Clear corrupted token data
    await clearRefreshToken();
    return null;
  }
}

/**
 * Clear stored refresh token and metadata
 *
 * Removes both the encrypted token and its associated metadata from
 * secure storage (Tauri) or localStorage. Used during logout or token reset operations.
 */
export async function clearRefreshToken(): Promise<void> {
  try {
    if (isTauriEnvironment()) {
      // Use Tauri secure storage
      await deleteTauriData(STORAGE_KEY);
      await deleteTauriData(`${STORAGE_KEY}_meta`);
      console.log("Refresh token and metadata cleared via Tauri");
    } else {
      // Fallback to localStorage for development
      localStorage.removeItem(STORAGE_KEY);
      localStorage.removeItem(`${STORAGE_KEY}_meta`);
      console.log("Refresh token and metadata cleared via localStorage");
    }
  } catch (error) {
    console.error("Failed to clear refresh token:", error);
    // Continue execution - storage cleanup is not critical
  }
}

/**
 * Save refresh token metadata for validation tracking
 *
 * Stores non-sensitive metadata about the refresh token such as validation
 * status and expiry information using Tauri secure storage or localStorage fallback.
 *
 * @param meta Token metadata object containing validation status and timestamps
 * @throws Error if metadata serialization or storage fails
 */
export async function saveRefreshTokenMetadata(
  meta: RefreshTokenStore
): Promise<void> {
  try {
    const metaJson = JSON.stringify(meta);

    if (isTauriEnvironment()) {
      // Use Tauri secure storage
      await storeTauriData(`${STORAGE_KEY}_meta`, metaJson);
      console.log("Token metadata saved via Tauri");
    } else {
      // Fallback to localStorage for development
      localStorage.setItem(`${STORAGE_KEY}_meta`, metaJson);
      console.log("Token metadata saved via localStorage");
    }
  } catch (error) {
    console.error("Failed to save token metadata:", error);
    throw new Error("Failed to save token metadata");
  }
}

/**
 * Load refresh token metadata
 *
 * Retrieves and parses stored token metadata from Tauri secure storage
 * or localStorage fallback. Returns null if no metadata exists or parsing fails.
 *
 * @returns Token metadata object or null if not found/invalid
 */
export async function loadRefreshTokenMetadata(): Promise<RefreshTokenStore | null> {
  try {
    let metaJson: string | null = null;

    if (isTauriEnvironment()) {
      // Use Tauri secure storage
      metaJson = await retrieveTauriData(`${STORAGE_KEY}_meta`);
    } else {
      // Fallback to localStorage for development
      metaJson = localStorage.getItem(`${STORAGE_KEY}_meta`);
    }

    return metaJson ? (JSON.parse(metaJson) as RefreshTokenStore) : null;
  } catch (error) {
    console.warn("Failed to load token metadata:", error);
    return null;
  }
}

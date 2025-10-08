/**
 * Crypto utilities for secure token handling
 * Uses Web Crypto API for encryption/decryption
 */

const ALGORITHM = "AES-GCM";
const KEY_LENGTH = 256;
const IV_LENGTH = 12;

export interface EncryptedData {
  data: string;
  iv: string;
}

/**
 * Derive a consistent key from a password/seed
 * @param password Base password/seed string
 * @returns CryptoKey derived from password
 */
async function deriveKeyFromPassword(password: string): Promise<CryptoKey> {
  const encoder = new TextEncoder();
  const keyMaterial = await window.crypto.subtle.importKey(
    "raw",
    encoder.encode(password),
    "PBKDF2",
    false,
    ["deriveBits", "deriveKey"]
  );

  return await window.crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: encoder.encode("blackbird-salt"), // Fixed salt for demo
      iterations: 100000,
      hash: "SHA-256",
    },
    keyMaterial,
    { name: ALGORITHM, length: KEY_LENGTH },
    false,
    ["encrypt", "decrypt"]
  );
}

/**
 * Encrypt a string using AES-GCM
 * @param plaintext String to encrypt
 * @param password Password for key derivation
 * @returns Base64 encoded encrypted data (IV + ciphertext)
 */
export async function encryptString(
  plaintext: string,
  password = "default-key"
): Promise<string> {
  const encoder = new TextEncoder();
  const key = await deriveKeyFromPassword(password);
  const iv = window.crypto.getRandomValues(new Uint8Array(IV_LENGTH));

  const encryptedBuffer = await window.crypto.subtle.encrypt(
    { name: ALGORITHM, iv },
    key,
    encoder.encode(plaintext)
  );

  // Combine IV + encrypted data
  const combined = new Uint8Array(iv.byteLength + encryptedBuffer.byteLength);
  combined.set(iv, 0);
  combined.set(new Uint8Array(encryptedBuffer), iv.byteLength);

  return btoa(String.fromCharCode(...combined));
}

/**
 * Decrypt a string using AES-GCM
 * @param encryptedBase64 Base64 encoded encrypted data (IV + ciphertext)
 * @param password Password for key derivation
 * @returns Decrypted plaintext string
 */
export async function decryptString(
  encryptedBase64: string,
  password = "default-key"
): Promise<string> {
  try {
    const decoder = new TextDecoder();
    const key = await deriveKeyFromPassword(password);

    // Decode from base64
    const combined = Uint8Array.from(atob(encryptedBase64), (c) =>
      c.charCodeAt(0)
    );

    // Extract IV and ciphertext
    const iv = combined.slice(0, IV_LENGTH);
    const ciphertext = combined.slice(IV_LENGTH);

    const decryptedBuffer = await window.crypto.subtle.decrypt(
      { name: ALGORITHM, iv },
      key,
      ciphertext
    );

    return decoder.decode(decryptedBuffer);
  } catch (error) {
    throw new Error(
      `Decryption failed: ${
        error instanceof Error ? error.message : "Unknown error"
      }`
    );
  }
}

/**
 * Generate a secure random string for use as encryption key
 * @param length Length of random string
 * @returns Random hex string
 */
export function generateSecureRandomString(length = 32): string {
  const array = new Uint8Array(length);
  window.crypto.getRandomValues(array);
  return Array.from(array, (byte) => byte.toString(16).padStart(2, "0")).join(
    ""
  );
}

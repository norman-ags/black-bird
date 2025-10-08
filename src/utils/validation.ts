/**
 * Validation utilities for parameter and type checking
 */

/**
 * Check if a value is a non-empty string
 * @param value Value to check
 * @param fieldName Field name for error messages
 * @returns true if valid
 * @throws Error if invalid
 */
export function validateRequiredString(
  value: unknown,
  fieldName: string
): asserts value is string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
}

/**
 * Check if a value is a valid refresh token format
 * @param token Token to validate
 * @returns true if valid format
 */
export function isValidRefreshTokenFormat(token: string): boolean {
  // Basic validation - should be a long string without obvious patterns
  return token.length >= 20 && /^[A-Za-z0-9._-]+$/.test(token);
}

/**
 * Validate email format
 * @param email Email to validate
 * @returns true if valid email format
 */
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

/**
 * Validate time format (HH:MM)
 * @param time Time string to validate
 * @returns true if valid 24-hour time format
 */
export function isValidTimeFormat(time: string): boolean {
  const timeRegex = /^([01]?[0-9]|2[0-3]):[0-5][0-9]$/;
  return timeRegex.test(time);
}

/**
 * Validate ISO date string
 * @param dateString Date string to validate
 * @returns true if valid ISO date
 */
export function isValidISODate(dateString: string): boolean {
  const date = new Date(dateString);
  return !isNaN(date.getTime()) && date.toISOString() === dateString;
}

/**
 * Sanitize string input to prevent basic injection attacks
 * @param input String to sanitize
 * @returns Sanitized string
 */
export function sanitizeString(input: string): string {
  return input
    .replace(/[<>]/g, "") // Remove angle brackets
    .replace(/javascript:/gi, "") // Remove javascript: protocol
    .trim();
}

/**
 * Validate that a number is within a specified range
 * @param value Number to validate
 * @param min Minimum value (inclusive)
 * @param max Maximum value (inclusive)
 * @param fieldName Field name for error messages
 * @returns true if valid
 * @throws Error if invalid
 */
export function validateNumberRange(
  value: number,
  min: number,
  max: number,
  fieldName: string
): void {
  if (typeof value !== "number" || isNaN(value)) {
    throw new Error(`${fieldName} must be a valid number`);
  }
  if (value < min || value > max) {
    throw new Error(`${fieldName} must be between ${min} and ${max}`);
  }
}

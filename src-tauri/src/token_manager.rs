/**
 * Universal Token Manager
 *
 * Implements the shared token logic pattern for all API operations:
 * 1. Try with saved access token first
 * 2. Only refresh on token-related errors (401, invalid_token, etc.)
 * 3. Single retry after token refresh
 * 4. Fixed storage keys that overwrite previous tokens
 */

use crate::errors::AppError;
use crate::storage::create_storage_backend;
use crate::commands::{exchange_refresh_token_api, TokenResponse};
use tauri::AppHandle;
use std::future::Future;

// Fixed storage keys - never change these
const REFRESH_TOKEN_KEY: &str = "refresh_token";
const ACCESS_TOKEN_KEY: &str = "access_token";

/// Check if an error is token-related (requires refresh)
pub fn is_token_error(error: &str) -> bool {
    error.contains("401") ||
    error.contains("unauthorized") ||
    error.contains("invalid_token") ||
    error.contains("token_expired") ||
    error.contains("Unauthorized") ||
    error.contains("Token") && error.contains("expired")
}

/// Get saved access token from storage
pub async fn get_saved_access_token(app_handle: &AppHandle) -> Result<String, AppError> {
    let storage = create_storage_backend(app_handle.clone())?;
    storage.retrieve(ACCESS_TOKEN_KEY).await?
        .ok_or_else(|| AppError::authentication("No access token found".to_string()))
}

/// Refresh tokens using saved refresh token and overwrite storage keys
pub async fn refresh_and_save_tokens(app_handle: &AppHandle) -> Result<TokenResponse, AppError> {
    let start_time = std::time::Instant::now();
    let storage = create_storage_backend(app_handle.clone())?;

    // Get current refresh token
    let refresh_token = storage.retrieve(REFRESH_TOKEN_KEY).await?
        .ok_or_else(|| AppError::authentication("No refresh token found".to_string()))?;

    // Exchange for new tokens
    let result = exchange_refresh_token_api(&refresh_token).await;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    match result {
        Ok(new_tokens) => {
            // OVERWRITE existing keys with new tokens (fixed key strategy)
            storage.store(REFRESH_TOKEN_KEY, &new_tokens.refresh_token).await?;
            storage.store(ACCESS_TOKEN_KEY, &new_tokens.access_token).await?;

            println!("[TokenManager] Tokens refreshed and saved successfully");

            // Log successful token refresh
            if let Some(logger) = crate::logging::get_logger() {
                let _ = logger.log_token_refresh(true, Some(duration_ms), None).await;
            }

            Ok(new_tokens)
        }
        Err(e) => {
            let error_msg = format!("Token refresh failed: {}", e);

            // Log failed token refresh
            if let Some(logger) = crate::logging::get_logger() {
                let _ = logger.log_token_refresh(false, Some(duration_ms), Some(&e)).await;
            }

            Err(AppError::authentication(error_msg))
        }
    }
}

/// Save initial tokens during setup (both refresh and access token)
pub async fn save_initial_tokens(
    app_handle: &AppHandle,
    refresh_token: &str,
    access_token: &str,
) -> Result<(), AppError> {
    let storage = create_storage_backend(app_handle.clone())?;

    // Store both tokens using fixed keys
    storage.store(REFRESH_TOKEN_KEY, refresh_token).await?;
    storage.store(ACCESS_TOKEN_KEY, access_token).await?;

    println!("[TokenManager] Initial tokens saved successfully");
    Ok(())
}

/// Universal API call pattern with shared token logic
///
/// This function implements the complete shared token flow:
/// 1. Try API call with saved access token
/// 2. If token error: refresh tokens and save to storage
/// 3. Retry API call once with new token
/// 4. If retry fails: return error (do nothing)
pub async fn api_with_shared_tokens<T, F, Fut>(
    app_handle: &AppHandle,
    operation: F,
    operation_name: &str,
) -> Result<T, AppError>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    // 1. Try with saved access token first
    let access_token = get_saved_access_token(app_handle).await?;

    match operation(access_token.clone()).await {
        Ok(result) => {
            println!("[TokenManager] {} succeeded with saved token", operation_name);
            Ok(result)
        }
        Err(error) if is_token_error(&error) => {
            println!("[TokenManager] {} failed with token error: {}", operation_name, error);
            println!("[TokenManager] Refreshing tokens and retrying...");

            // 2. Token error: refresh and save tokens
            match refresh_and_save_tokens(app_handle).await {
                Ok(new_tokens) => {
                    // 3. Retry once with new token
                    match operation(new_tokens.access_token).await {
                        Ok(result) => {
                            println!("[TokenManager] {} retry succeeded", operation_name);
                            Ok(result)
                        }
                        Err(retry_error) => {
                            println!("[TokenManager] {} retry failed: {}", operation_name, retry_error);
                            Err(AppError::api(format!("{} retry failed: {}", operation_name, retry_error), Some(500)))
                        }
                    }
                }
                Err(refresh_error) => {
                    println!("[TokenManager] Token refresh failed: {}", refresh_error);
                    Err(refresh_error)
                }
            }
        }
        Err(error) => {
            println!("[TokenManager] {} failed with non-token error: {}", operation_name, error);
            Err(AppError::api(format!("{} failed: {}", operation_name, error), Some(500)))
        }
    }
}

/// Wrapper for attendance API using shared token logic
pub async fn attendance_check_with_shared_tokens(
    app_handle: &AppHandle,
) -> Result<Option<crate::commands::AttendanceItem>, AppError> {
    let start_time = std::time::Instant::now();

    let result = api_with_shared_tokens(
        app_handle,
        |token| async move {
            crate::commands::get_attendance_status_api(&token).await
        },
        "attendance_check",
    ).await;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Log attendance check
    if let Some(logger) = crate::logging::get_logger() {
        match &result {
            Ok(_) => {
                let _ = logger.log_attendance_check(true, Some(duration_ms), None).await;
            }
            Err(e) => {
                let _ = logger.log_attendance_check(false, Some(duration_ms), Some(&e.to_string())).await;
            }
        }
    }

    result
}

/// Wrapper for clock-in API using shared token logic
pub async fn clock_in_with_shared_tokens(app_handle: &AppHandle) -> Result<bool, AppError> {
    let start_time = std::time::Instant::now();

    let result = api_with_shared_tokens(
        app_handle,
        |token| async move {
            crate::commands::clock_in_api(&token).await
        },
        "clock_in",
    ).await;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Log clock-in operation
    if let Some(logger) = crate::logging::get_logger() {
        match &result {
            Ok(true) => {
                let _ = logger.log_clock_in(true, "shared_token_api", Some(duration_ms), None).await;
            }
            Ok(false) => {
                let _ = logger.log_clock_in(false, "shared_token_api", Some(duration_ms), Some("API returned false")).await;
            }
            Err(e) => {
                let _ = logger.log_clock_in(false, "shared_token_api", Some(duration_ms), Some(&e.to_string())).await;
            }
        }
    }

    result
}

/// Wrapper for clock-out API using shared token logic
pub async fn clock_out_with_shared_tokens(app_handle: &AppHandle) -> Result<bool, AppError> {
    let start_time = std::time::Instant::now();

    let result = api_with_shared_tokens(
        app_handle,
        |token| async move {
            crate::commands::clock_out_api(&token).await
        },
        "clock_out",
    ).await;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Log clock-out operation
    if let Some(logger) = crate::logging::get_logger() {
        match &result {
            Ok(true) => {
                let _ = logger.log_clock_out(true, "shared_token_api", Some(duration_ms), None).await;
            }
            Ok(false) => {
                let _ = logger.log_clock_out(false, "shared_token_api", Some(duration_ms), Some("API returned false")).await;
            }
            Err(e) => {
                let _ = logger.log_clock_out(false, "shared_token_api", Some(duration_ms), Some(&e.to_string())).await;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_token_error() {
        assert!(is_token_error("401 Unauthorized"));
        assert!(is_token_error("Token expired"));
        assert!(is_token_error("invalid_token"));
        assert!(is_token_error("unauthorized"));

        assert!(!is_token_error("500 Internal Server Error"));
        assert!(!is_token_error("Network connection failed"));
        assert!(!is_token_error("Parse error"));
    }
}
//! Password hashing utilities using Argon2id.
//!
//! This module provides non-blocking password hashing and verification functions
//! that can be used in async contexts without blocking the tokio runtime.

use crate::errors::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a password using Argon2id with default parameters.
///
/// This function uses `tokio::task::spawn_blocking` to avoid blocking the async runtime,
/// as Argon2 is CPU-bound.
///
/// # Arguments
/// * `password` - The plaintext password to hash
///
/// # Returns
/// * `Ok(String)` - The hashed password in PHC string format
/// * `Err(AppError)` - If hashing fails
pub async fn hash_password(password: &str) -> Result<String, AppError> {
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| {
                tracing::error!("Failed to hash password: {}", e);
                AppError::internal_error("Failed to hash password")
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("Task join error for password hashing: {}", e);
        AppError::internal_error("Failed to spawn task to hash password")
    })?
}

/// Verify a password against a stored hash using Argon2id.
///
/// This function uses `tokio::task::spawn_blocking` to avoid blocking the async runtime.
///
/// # Arguments
/// * `password` - The plaintext password to verify
/// * `hash` - The stored PHC-formatted hash to verify against
///
/// # Returns
/// * `Ok(true)` - If the password matches the hash
/// * `Ok(false)` - If the password does not match
/// * `Err(AppError)` - If verification fails due to invalid hash format
pub async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    // Run blocking password verification in a separate thread
    let password = password.to_string();
    let hash = hash.to_string();

    tokio::task::spawn_blocking(move || {
        let argon2 = Argon2::default();

        let parsed_hash = PasswordHash::new(&hash).map_err(|e| {
            tracing::error!("Invalid password hash format: {}", e);
            AppError::internal_error("Invalid password hash format")
        })?;

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    })
    .await
    .map_err(|e| {
        tracing::error!("Task join error for password verification: {}", e);
        AppError::internal_error("Failed to verify password")
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_password_generates_valid_hash() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).await.unwrap();

        // Verify the hash is in PHC format (starts with $argon2)
        assert!(hash.starts_with("$argon2"));
    }

    #[tokio::test]
    async fn test_verify_password_correct_password() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).await.unwrap();

        let result = verify_password(password, &hash).await.unwrap();
        assert!(result, "Verification should succeed with correct password");
    }

    #[tokio::test]
    async fn test_verify_password_wrong_password() {
        let password = "CorrectPassword";
        let wrong_password = "WrongPassword";
        let hash = hash_password(password).await.unwrap();

        let result = verify_password(wrong_password, &hash).await.unwrap();
        assert!(!result, "Verification should fail with wrong password");
    }

    #[tokio::test]
    async fn test_verify_password_malformed_hash() {
        let password = "SomePassword";
        let malformed_hash = "not-a-valid-hash";

        let result = verify_password(password, malformed_hash).await;
        assert!(
            result.is_err(),
            "Verification should fail with malformed hash"
        );
    }

    #[tokio::test]
    async fn test_hash_is_different_each_time() {
        let password = "SamePassword123!";

        let hash1 = hash_password(password).await.unwrap();
        let hash2 = hash_password(password).await.unwrap();

        // Hashes should be different due to unique salts
        assert_ne!(hash1, hash2);

        // But both should verify successfully
        assert!(verify_password(password, &hash1).await.unwrap());
        assert!(verify_password(password, &hash2).await.unwrap());
    }
}

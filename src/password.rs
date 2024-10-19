use bcrypt::{hash, verify, DEFAULT_COST};
use dialoguer::{theme::ColorfulTheme, Password};

use crate::error::LockerError;

pub fn get_password() -> Result<String, LockerError> {
    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a password to lock the folder")
        .with_confirmation("Confirm password", "Passwords don't match")
        .interact()
        .map_err(|_| LockerError::PasswordOperationFailed {
            operation: "input".to_string(),
            reason: "User interaction error".to_string(),
        })?;

    Ok(password)
}

pub fn hash_password(password: &str) -> Result<String, LockerError> {
    hash(password, DEFAULT_COST).map_err(|_| LockerError::PasswordOperationFailed {
        operation: "hash".to_string(),
        reason: "Encryption error".to_string(),
    })
}

pub fn verify_password(input: &str, stored_password: &str) -> Result<bool, LockerError> {
    verify(input, stored_password).map_err(|_| LockerError::PasswordOperationFailed {
        operation: "verify".to_string(),
        reason: "Authentication error".to_string(),
    })
}

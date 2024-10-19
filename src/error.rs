use std::path::PathBuf;
use thiserror::Error;

/// Represents errors that can occur during folder locking and unlocking operations.
#[derive(Error, Debug)]
pub enum LockerError {
    /// The provided folder name is invalid or cannot be processed.
    #[error("Invalid folder name: Unable to process the given folder")]
    InvalidFolderName,

    /// The provided password is incorrect or does not match the stored password.
    #[error("Invalid password: Authentication failed")]
    InvalidPassword,

    /// The folder is not locked.
    #[error("Folder is not locked")]
    FolderNotLocked,

    /// Failed to perform a file or folder operation at the specified path.
    #[error("Failed to {operation} at {path}: {error}")]
    FileOperationFailed {
        operation: String,
        path: PathBuf,
        error: String,
    },

    /// Failed to perform a password-related operation.
    #[error("Failed to {operation} password: {reason}")]
    PasswordOperationFailed { operation: String, reason: String },

    /// An I/O error occurred during file or folder operations.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error occurred during bcrypt operations.
    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
}

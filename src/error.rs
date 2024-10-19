use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LockerError {
    #[error("Invalid folder name")]
    InvalidFolderName,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Failed to create locker folder: {0}")]
    LockerCreationFailed(PathBuf),

    #[error("Failed to write metadata file: {0}")]
    MetadataWriteFailed(PathBuf),

    #[error("Failed to read metadata file: {0}")]
    MetadataReadFailed(PathBuf),

    #[error("Failed to remove metadata file: {0}")]
    MetadataRemoveFailed(PathBuf),

    #[error("Failed to get password input")]
    PasswordInputFailed,

    #[error("Failed to hash password")]
    PasswordHashFailed,

    #[error("Failed to verify password")]
    PasswordVerificationFailed,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
}

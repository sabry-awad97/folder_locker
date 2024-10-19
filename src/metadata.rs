use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use colored::Colorize;
use log::error;

use crate::error::LockerError;
use crate::file_manager::FileManager;
pub const METADATA_FILE: &str = ".locker_metadata";

pub fn write_metadata(hidden_path: &Path, hashed_password: &str) -> Result<(), LockerError> {
    let metadata_path = hidden_path.join(METADATA_FILE);
    let mut file = File::create(&metadata_path).map_err(|e| LockerError::FileOperationFailed {
        operation: "create".to_string(),
        path: metadata_path.clone(),
        error: e.to_string(),
    })?;
    file.write_all(hashed_password.as_bytes())
        .map_err(|e| LockerError::FileOperationFailed {
            operation: "write".to_string(),
            path: metadata_path.clone(),
            error: e.to_string(),
        })?;
    if let Err(e) = FileManager::set_file_attributes(&metadata_path) {
        error!("Failed to set file attributes: {}", e);
        println!("{}", "Failed to set file attributes.".red());
    };
    Ok(())
}

pub fn read_metadata(hidden_path: &Path) -> Result<String, LockerError> {
    let metadata_path = hidden_path.join(METADATA_FILE);
    let mut stored_password = String::new();
    File::open(&metadata_path)
        .and_then(|mut file| file.read_to_string(&mut stored_password))
        .map_err(|e| LockerError::FileOperationFailed {
            operation: "read".to_string(),
            path: metadata_path.clone(),
            error: e.to_string(),
        })?;
    Ok(stored_password)
}

pub fn remove_metadata(hidden_path: &Path) -> Result<(), LockerError> {
    if let Err(e) = FileManager::remove_folder_attributes(hidden_path.to_str().unwrap()) {
        error!("Failed to remove folder attributes: {}", e);
        println!("{}", "Failed to remove folder attributes.".red());
    };
    let metadata_path = hidden_path.join(METADATA_FILE);
    std::fs::remove_file(metadata_path).map_err(|e| LockerError::FileOperationFailed {
        operation: "remove".to_string(),
        path: hidden_path.join(METADATA_FILE).clone(),
        error: e.to_string(),
    })?;
    Ok(())
}

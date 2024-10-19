use bcrypt::{hash, verify, DEFAULT_COST};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Password};
use log::{error, info, warn};
use std::io::Write;
use std::path::Path;
use std::{fs, fs::File, io::Read};

use crate::error::LockerError;
use crate::utils::{remove_folder_attributes, set_file_attributes, set_folder_attributes};

const METADATA_FILE: &str = ".locker_metadata";

pub fn lock_folder(folder: Option<&Path>) -> Result<(), LockerError> {
    let folder_path = folder.unwrap_or_else(|| Path::new("."));
    let folder_name = folder_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(LockerError::InvalidFolderName)?;
    let hidden_path = folder_path.with_file_name(format!(".{}", folder_name));

    if hidden_path.exists() {
        warn!("Folder is already locked: {:?}", hidden_path);
        println!("{}", "Folder is already locked.".yellow());
        return Ok(());
    }

    let password = get_password()?;
    let hashed_password = hash_password(&password)?;

    fs::create_dir(&hidden_path).map_err(|e| LockerError::FileOperationFailed {
        operation: "create".to_string(),
        path: hidden_path.clone(),
        error: e.to_string(),
    })?;
    info!("Locker folder created: {:?}", hidden_path);
    println!("{}", "Locker folder created.".green());

    write_metadata(&hidden_path, &hashed_password)?;
    set_folder_attributes(hidden_path.to_str().unwrap());

    info!("Folder locked successfully: {:?}", hidden_path);
    println!("{}", "Folder locked successfully.".green().bold());
    Ok(())
}

pub fn unlock_folder(folder: Option<&Path>) -> Result<(), LockerError> {
    let folder_path = folder.unwrap_or_else(|| Path::new("."));
    let folder_name = folder_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(LockerError::InvalidFolderName)?;
    let hidden_path = folder_path.with_file_name(format!(".{}", folder_name));

    if !hidden_path.exists() {
        warn!("No locked folder found: {:?}", hidden_path);
        println!("{}", "No locked folder found.".yellow());
        return Ok(());
    }

    let input = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter password")
        .interact()
        .map_err(|_| LockerError::PasswordOperationFailed {
            operation: "input".to_string(),
            reason: "User interaction error".to_string(),
        })?;

    let stored_password = read_metadata(&hidden_path)?;

    if !verify_password(&input, &stored_password)? {
        error!("Invalid password attempt for folder: {:?}", hidden_path);
        println!("{}", "Invalid password!".red().bold());
        return Err(LockerError::InvalidPassword);
    }

    remove_folder_attributes(hidden_path.to_str().unwrap());
    fs::remove_file(hidden_path.join(METADATA_FILE)).map_err(|e| {
        LockerError::FileOperationFailed {
            operation: "remove".to_string(),
            path: hidden_path.join(METADATA_FILE).clone(),
            error: e.to_string(),
        }
    })?;

    info!("Folder unlocked successfully: {:?}", hidden_path);
    println!("{}", "Folder unlocked successfully.".green().bold());
    Ok(())
}

// Helper functions

fn get_password() -> Result<String, LockerError> {
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

fn hash_password(password: &str) -> Result<String, LockerError> {
    hash(password, DEFAULT_COST).map_err(|_| LockerError::PasswordOperationFailed {
        operation: "hash".to_string(),
        reason: "Encryption error".to_string(),
    })
}

fn verify_password(input: &str, stored_password: &str) -> Result<bool, LockerError> {
    verify(input, stored_password).map_err(|_| LockerError::PasswordOperationFailed {
        operation: "verify".to_string(),
        reason: "Authentication error".to_string(),
    })
}

fn write_metadata(hidden_path: &Path, hashed_password: &str) -> Result<(), LockerError> {
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
    set_file_attributes(&metadata_path);
    Ok(())
}

fn read_metadata(hidden_path: &Path) -> Result<String, LockerError> {
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

use colored::*;
use dialoguer::{theme::ColorfulTheme, Password};
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::LockerError;
use crate::metadata::{read_metadata, remove_metadata, write_metadata};
use crate::password::{get_password, hash_password, verify_password};
use crate::permission_manager::PermissionManager;

// New reusable functions
fn get_folder_paths(folder: Option<&Path>) -> Result<(PathBuf, PathBuf), LockerError> {
    let folder_path = folder.unwrap_or_else(|| Path::new("."));
    let folder_name = folder_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(LockerError::InvalidFolderName)?;
    let hidden_path = folder_path.with_file_name(format!(".{}", folder_name));
    Ok((folder_path.to_path_buf(), hidden_path))
}

fn check_folder_status(hidden_path: &Path, is_locking: bool) -> Result<(), LockerError> {
    let (exists, _action, status) = if is_locking {
        (hidden_path.exists(), "lock", "already locked")
    } else {
        (!hidden_path.exists(), "unlock", "not locked")
    };

    if exists {
        warn!("Folder is {}: {:?}", status, hidden_path);
        println!("{}", format!("Folder is {}.", status).yellow());
        return Ok(());
    }

    if !is_locking && !hidden_path.exists() {
        error!("Cannot unlock folder: {:?} is not locked", hidden_path);
        println!("{}", "Folder is not locked.".red());
        return Err(LockerError::FolderNotLocked);
    }

    Ok(())
}

pub fn lock_folder(folder: Option<&Path>) -> Result<(), LockerError> {
    let (_folder_path, hidden_path) = get_folder_paths(folder)?;
    check_folder_status(&hidden_path, true)?;

    let password = get_password()?;
    let hashed_password = hash_password(&password)?;

    create_folder(&hidden_path)?;

    write_metadata(&hidden_path, &hashed_password)?;

    if let Err(e) = PermissionManager::set_attributes(hidden_path.to_str().unwrap()) {
        error!("Failed to set folder attributes: {}", e);
    };

    info!("Folder locked successfully: {:?}", hidden_path);
    println!("{}", "Folder locked successfully.".green().bold());
    Ok(())
}

pub fn unlock_folder(folder: Option<&Path>) -> Result<(), LockerError> {
    let (folder_path, hidden_path) = get_folder_paths(folder)?;
    check_folder_status(&hidden_path, false)?;

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

    if let Err(e) = PermissionManager::remove_attributes(hidden_path.to_str().unwrap()) {
        error!("Failed to allow folder deletion: {}", e);
    };

    remove_metadata(&hidden_path)?;

    // Remove the leading dot from the folder name
    let unlocked_path =
        folder_path.with_file_name(folder_path.file_name().unwrap().to_str().unwrap());
    rename_folder(&hidden_path, &unlocked_path)?;

    info!("Folder unlocked successfully: {:?}", unlocked_path);
    println!("{}", "Folder unlocked successfully.".green().bold());
    Ok(())
}

fn create_folder(path: &Path) -> Result<(), LockerError> {
    fs::create_dir(path).map_err(|e| LockerError::FileOperationFailed {
        operation: "create".to_string(),
        path: path.to_path_buf(),
        error: e.to_string(),
    })?;
    info!("Locker folder created: {:?}", path);
    println!("{}", "Locker folder created.".green());
    Ok(())
}

fn rename_folder(from: &Path, to: &Path) -> Result<(), LockerError> {
    fs::rename(from, to).map_err(|e| LockerError::FileOperationFailed {
        operation: "rename".to_string(),
        path: from.to_path_buf(),
        error: e.to_string(),
    })?;
    Ok(())
}

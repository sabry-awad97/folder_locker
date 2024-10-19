use colored::*;
use dialoguer::{theme::ColorfulTheme, Password};
use log::{error, info, warn};
use std::fs;
use std::path::Path;

use crate::error::LockerError;
use crate::metadata::{read_metadata, remove_metadata, write_metadata};
use crate::password::{get_password, hash_password, verify_password};
use crate::utils::set_folder_attributes;

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

    remove_metadata(&hidden_path)?;

    info!("Folder unlocked successfully: {:?}", hidden_path);
    println!("{}", "Folder unlocked successfully.".green().bold());
    Ok(())
}

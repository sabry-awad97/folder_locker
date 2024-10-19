use colored::*;
use dialoguer::{theme::ColorfulTheme, Password};
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::LockerError;
use crate::metadata::{read_metadata, remove_metadata, write_metadata};
use crate::password::{get_password, hash_password, verify_password};
use crate::permission_manager::PermissionManager;

// New struct to encapsulate folder operations
struct FolderOperator {
    folder_path: PathBuf,
    hidden_path: PathBuf,
}

impl FolderOperator {
    fn new(folder: Option<&Path>) -> Result<Self, LockerError> {
        let (folder_path, hidden_path) = Self::get_folder_paths(folder)?;
        Ok(Self {
            folder_path,
            hidden_path,
        })
    }

    fn get_folder_paths(folder: Option<&Path>) -> Result<(PathBuf, PathBuf), LockerError> {
        let folder_path = folder.unwrap_or_else(|| Path::new("."));
        let folder_name = folder_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(LockerError::InvalidFolderName)?;
        let hidden_path = folder_path.with_file_name(format!(".{}", folder_name));
        Ok((folder_path.to_path_buf(), hidden_path))
    }

    fn check_folder_status(&self, is_locking: bool) -> Result<(), LockerError> {
        let (exists, _action, status) = if is_locking {
            (self.hidden_path.exists(), "lock", "already locked")
        } else {
            (!self.hidden_path.exists(), "unlock", "not locked")
        };

        if exists {
            warn!("Folder is {}: {:?}", status, self.hidden_path);
            println!("{}", format!("Folder is {}.", status).yellow());
            return Ok(());
        }

        if !is_locking && !self.hidden_path.exists() {
            error!("Cannot unlock folder: {:?} is not locked", self.hidden_path);
            println!("{}", "Folder is not locked.".red());
            return Err(LockerError::FolderNotLocked);
        }

        Ok(())
    }

    fn lock(&self) -> Result<(), LockerError> {
        self.check_folder_status(true)?;

        let password = get_password()?;
        let hashed_password = hash_password(&password)?;

        if self.folder_path.exists() {
            self.rename_folder(&self.folder_path, &self.hidden_path)?;
        } else {
            self.create_folder(&self.hidden_path)?;
        }

        write_metadata(&self.hidden_path, &hashed_password)?;

        PermissionManager::set_attributes(self.hidden_path.to_str().unwrap())?;

        info!("Folder locked successfully: {:?}", self.hidden_path);
        println!("{}", "Folder locked successfully.".green().bold());
        Ok(())
    }

    fn unlock(&self) -> Result<(), LockerError> {
        self.check_folder_status(false)?;

        let input = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter password")
            .interact()
            .map_err(|_| LockerError::PasswordOperationFailed {
                operation: "input".to_string(),
                reason: "User interaction error".to_string(),
            })?;

        let stored_password = read_metadata(&self.hidden_path)?;

        if !verify_password(&input, &stored_password)? {
            error!(
                "Invalid password attempt for folder: {:?}",
                self.hidden_path
            );
            println!("{}", "Invalid password!".red().bold());
            return Err(LockerError::InvalidPassword);
        }

        PermissionManager::remove_attributes(self.hidden_path.to_str().unwrap())?;

        remove_metadata(&self.hidden_path)?;

        let unlocked_path = self
            .folder_path
            .with_file_name(self.folder_path.file_name().unwrap().to_str().unwrap());
        self.rename_folder(&self.hidden_path, &unlocked_path)?;

        info!("Folder unlocked successfully: {:?}", unlocked_path);
        println!("{}", "Folder unlocked successfully.".green().bold());
        Ok(())
    }

    fn create_folder(&self, path: &Path) -> Result<(), LockerError> {
        fs::create_dir(path).map_err(|e| LockerError::FileOperationFailed {
            operation: "create".to_string(),
            path: path.to_path_buf(),
            error: e.to_string(),
        })?;
        info!("Locker folder created: {:?}", path);
        println!("{}", "Locker folder created.".green());
        Ok(())
    }

    fn rename_folder(&self, from: &Path, to: &Path) -> Result<(), LockerError> {
        fs::rename(from, to).map_err(|e| LockerError::FileOperationFailed {
            operation: "rename".to_string(),
            path: from.to_path_buf(),
            error: e.to_string(),
        })?;
        Ok(())
    }
}

pub fn lock_folder(folder: Option<&Path>) -> Result<(), LockerError> {
    FolderOperator::new(folder)?.lock()
}

pub fn unlock_folder(folder: Option<&Path>) -> Result<(), LockerError> {
    FolderOperator::new(folder)?.unlock()
}

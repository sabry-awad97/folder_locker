use colored::*;
use dialoguer::{theme::ColorfulTheme, Password};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::error::LockerError;
use crate::metadata::{read_metadata, remove_metadata, write_metadata};
use crate::password::{get_password, hash_password, verify_password};
use crate::permission_manager::PermissionManager;

// Add derive macros for better debugging and cloning capabilities
#[derive(Debug, Clone)]
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
        let spinner = self.create_spinner("Checking folder status...");

        let (exists, status) = if is_locking {
            (self.hidden_path.exists(), "already locked")
        } else {
            (!self.hidden_path.exists(), "not locked")
        };

        if exists {
            spinner.finish_with_message(format!("Folder is {}.", status));
            warn!("Folder is {}: {:?}", status, self.hidden_path);
            println!("{}", format!("Folder is {}.", status).yellow());
            return Ok(());
        }

        if !is_locking && !self.hidden_path.exists() {
            spinner.finish_with_message("Folder is not locked.");
            error!("Cannot unlock folder: {:?} is not locked", self.hidden_path);
            println!("{}", "Folder is not locked.".red());
            return Err(LockerError::FolderNotLocked);
        }

        spinner.finish_with_message("Folder status check complete.");
        Ok(())
    }

    fn lock(&self) -> Result<(), LockerError> {
        self.check_folder_status(true)?;

        let password = get_password()?;
        let hashed_password = hash_password(&password)?;

        let pb = self.create_progress_bar(4);

        self.perform_lock_steps(&pb, &hashed_password)?;

        pb.finish_with_message("Folder locked successfully!");

        info!("Folder locked successfully: {:?}", self.hidden_path);
        println!("{}", "Folder locked successfully.".green().bold());
        Ok(())
    }

    fn unlock(&self) -> Result<(), LockerError> {
        self.check_folder_status(false)?;
        self.verify_password()?;

        println!("{}", "Password verified successfully!".green());

        let pb = self.create_progress_bar(3);

        self.perform_unlock_steps(&pb)?;

        pb.finish_with_message("Folder unlocked successfully!");

        info!("Folder unlocked successfully: {:?}", self.folder_path);
        println!("{}", "Folder unlocked successfully.".green().bold());
        Ok(())
    }

    // Helper methods to improve readability and maintainability

    fn create_spinner(&self, message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.set_message(message.to_string());
        pb
    }

    fn create_progress_bar(&self, steps: u64) -> ProgressBar {
        let pb = ProgressBar::new(steps);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    }

    fn perform_lock_steps(
        &self,
        pb: &ProgressBar,
        hashed_password: &str,
    ) -> Result<(), LockerError> {
        let spinner = self.create_spinner("Preparing to lock folder...");
        thread::sleep(Duration::from_secs(1));
        spinner.finish_and_clear();
        pb.inc(1);

        if self.folder_path.exists() {
            let spinner = self.create_spinner("Renaming folder...");
            self.rename_folder(&self.folder_path, &self.hidden_path)?;
            thread::sleep(Duration::from_secs(1));
            spinner.finish_and_clear();
        } else {
            let spinner = self.create_spinner("Creating hidden folder...");
            self.create_folder(&self.hidden_path)?;
            thread::sleep(Duration::from_secs(1));
            spinner.finish_and_clear();
        }
        pb.inc(1);

        let spinner = self.create_spinner("Writing metadata...");
        write_metadata(&self.hidden_path, hashed_password)?;
        thread::sleep(Duration::from_secs(1));
        spinner.finish_and_clear();
        pb.inc(1);

        let spinner = self.create_spinner("Setting folder attributes...");
        PermissionManager::set_attributes(self.hidden_path.to_str().unwrap())?;
        thread::sleep(Duration::from_secs(1));
        spinner.finish_and_clear();
        pb.inc(1);

        Ok(())
    }

    fn verify_password(&self) -> Result<(), LockerError> {
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

        Ok(())
    }

    fn perform_unlock_steps(&self, pb: &ProgressBar) -> Result<(), LockerError> {
        let spinner = self.create_spinner("Removing folder attributes...");
        PermissionManager::remove_attributes(self.hidden_path.to_str().unwrap())?;
        thread::sleep(Duration::from_secs(1));
        spinner.finish_and_clear();
        pb.inc(1);

        let spinner = self.create_spinner("Removing metadata...");
        remove_metadata(&self.hidden_path)?;
        thread::sleep(Duration::from_secs(1));
        spinner.finish_and_clear();
        pb.inc(1);

        let spinner = self.create_spinner("Renaming folder...");
        let unlocked_path = self
            .folder_path
            .with_file_name(self.folder_path.file_name().unwrap().to_str().unwrap());
        self.rename_folder(&self.hidden_path, &unlocked_path)?;
        thread::sleep(Duration::from_secs(1));
        spinner.finish_and_clear();
        pb.inc(1);

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

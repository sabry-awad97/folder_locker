use crate::{
    error::LockerError,
    folder_operations::{lock_folder, unlock_folder},
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Command-line interface for the folder locker application
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The action to perform on the folder
    #[clap(subcommand)]
    pub action: Option<Action>,
}

/// Available actions for the folder locker
#[derive(Subcommand, Debug)]
pub enum Action {
    /// Lock the folder
    Lock {
        /// Path to the folder to lock
        #[clap(value_parser)]
        folder: Option<PathBuf>,
    },
    /// Unlock the folder
    Unlock {
        /// Path to the folder to unlock
        #[clap(value_parser)]
        folder: Option<PathBuf>,
    },
}

impl Args {
    /// Create a new instance of Args
    pub fn new() -> Self {
        Self::parse()
    }
}

impl Action {
    /// Execute the selected action
    pub fn execute(&self) -> Result<(), LockerError> {
        match self {
            Action::Lock { folder } => lock_folder(folder.as_deref()),
            Action::Unlock { folder } => unlock_folder(folder.as_deref()),
        }
    }
}

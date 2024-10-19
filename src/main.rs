mod cli;
mod error;
mod folder_operations;
mod utils;
mod metadata;
mod password;

use crate::cli::Args;
use colored::Colorize;
use log::{error, info};

fn main() {
    env_logger::init();
    info!("Starting folder locker application");

    let args = Args::new();

    match args.action {
        Some(action) => {
            info!("Executing action: {:?}", action);
            if let Err(e) = action.execute() {
                error!("Error executing action: {}", e);
                eprintln!("{}", format!("Error: {}", e).red().bold());
            }
        }
        None => {
            info!("No action specified");
            println!(
                "{}",
                "No action specified. Use --help for more information.".yellow()
            );
        }
    }

    info!("Folder locker application finished");
}

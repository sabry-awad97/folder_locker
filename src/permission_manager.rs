use std::io;
use std::path::Path;
use std::process::Command;

pub struct PermissionManager;

impl PermissionManager {
    pub fn set_file_attributes(path: &Path) -> io::Result<()> {
        Self::icacls(&[
            path.to_str().unwrap(),
            "/inheritance:d",
            "/grant:r",
            "Administrators:(OI)(CI)F",
            "/remove",
            "*S-1-1-0",
            "/deny",
            "*S-1-1-0:(DE,DC)",
        ])
    }

    pub fn set_folder_attributes(name: &str) -> io::Result<()> {
        Self::icacls(&[
            name,
            "/inheritance:d",
            "/grant:r",
            "Administrators:(OI)(CI)F",
            "/remove",
            "*S-1-1-0",
            "/deny",
            "*S-1-1-0:(DE,DC)",
        ])
    }

    pub fn remove_folder_attributes(name: &str) -> io::Result<()> {
        Self::icacls(&[name, "/reset", "/T"])
    }

    pub fn prevent_folder_deletion(folder_path: &str) -> io::Result<()> {
        Self::icacls(&[folder_path, "/deny", "*S-1-1-0:(DE,DC)"])
    }

    pub fn allow_folder_deletion(folder_path: &str) -> io::Result<()> {
        println!("Attempting to allow folder deletion for: {}", folder_path);
        Self::icacls(&[folder_path, "/reset", "/T"])
    }

    fn icacls(args: &[&str]) -> io::Result<()> {
        let output = Command::new("icacls").args(args).output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ))
        }
    }
}

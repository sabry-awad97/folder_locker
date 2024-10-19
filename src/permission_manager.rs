use std::io;
use std::path::Path;
use std::process::Command;

pub struct PermissionManager;

impl PermissionManager {
    /// Sets specific attributes on a file or folder to restrict access.
    ///
    /// This function modifies the access control list (ACL) of the specified path
    /// to enhance security and restrict access. Here's what each argument does:
    ///
    /// - `/inheritance:d`: Disables inheritance from parent objects.
    /// - `/grant:r`: Grants read-only access.
    /// - `Administrators:(OI)(CI)F`: Gives full control to Administrators, applying to this object and child objects.
    /// - `/remove *S-1-1-0`: Removes permissions for the "Everyone" group (SID S-1-1-0).
    /// - `/deny *S-1-1-0:(DE,DC)`: Denies delete and change permissions to the "Everyone" group.
    ///
    /// # Arguments
    ///
    /// * `path` - A path-like object representing the file or folder to modify.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>` - Ok if successful, Err with io::Error if failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid or if the `icacls` command fails.
    pub fn set_attributes<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;

        Self::icacls(&[
            path_str,
            "/inheritance:d",
            "/grant:r",
            "Administrators:(OI)(CI)F",
            "/remove",
            "*S-1-1-0",
            "/deny",
            "*S-1-1-0:(DE,DC)",
        ])
    }

    /// Removes custom attributes and resets permissions on a file or folder.
    ///
    /// This function uses the `icacls` command to reset the access control lists (ACLs)
    /// on the specified file or folder to their inherited values. It effectively
    /// removes any custom permissions that were previously set.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the path of the file or folder.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>` - Ok if the operation was successful, Err otherwise.
    ///
    /// # How it works
    ///
    /// 1. Calls `Self::icacls` with the following arguments:
    ///    - The path of the file or folder (`name`)
    ///    - `/reset`: Resets the ACLs to inherited ACLs
    ///    - `/T`: Applies the operation to all subfolders and files (for directories)
    ///
    /// This effectively undoes the changes made by `set_attributes`, restoring
    /// default permissions and allowing normal access to the file or folder.
    pub fn remove_attributes(name: &str) -> io::Result<()> {
        Self::icacls(&[name, "/reset", "/T"])
    }

    /// Executes the `icacls` command with the given arguments.
    ///
    /// This function is used to modify discretionary access control lists (DACLs) on files and folders
    /// in Windows systems. It runs the `icacls` (Improved Command Access Control Lists) command-line tool
    /// with the provided arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A slice of string slices containing the arguments to pass to `icacls`.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command executes successfully.
    /// * `Err(io::Error)` if the command fails, containing the error message from stderr.
    ///
    /// # How it works
    ///
    /// 1. Creates a new `Command` for "icacls" with the given arguments.
    /// 2. Executes the command and captures its output.
    /// 3. Checks if the command was successful (exit status 0).
    /// 4. If successful, returns `Ok(())`.
    /// 5. If unsuccessful, returns an `Err` with the error message from stderr.
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

use native_windows_gui as nwg;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::rc::Rc;
use winapi::shared::minwindef::DWORD;
use winapi::um::fileapi::SetFileAttributesW;
use winapi::um::winbase::MoveFileExW;
use winapi::um::winbase::MOVEFILE_REPLACE_EXISTING;
use winapi::um::winnt::{
    DELETE, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_SYSTEM,
};
use windows_acl::acl::ACL;
use windows_sys::Win32::Security::{CreateWellKnownSid, WinWorldSid, PSID};

pub struct FileManager;

impl FileManager {
    pub fn set_file_attributes(path: &Path) -> Result<(), DWORD> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let result = unsafe {
            SetFileAttributesW(wide.as_ptr(), FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM)
        };
        if result == 0 {
            Err(unsafe { winapi::um::errhandlingapi::GetLastError() })
        } else {
            Ok(())
        }
    }

    pub fn set_folder_attributes(name: &str) -> Result<(), DWORD> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = std::ffi::OsStr::new(name)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let result = unsafe {
            SetFileAttributesW(
                wide.as_ptr(),
                FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY,
            )
        };
        if result == 0 {
            Err(unsafe { winapi::um::errhandlingapi::GetLastError() })
        } else {
            Ok(())
        }
    }

    pub fn remove_folder_attributes(name: &str) -> Result<(), DWORD> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = std::ffi::OsStr::new(name)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let result = unsafe { SetFileAttributesW(wide.as_ptr(), 0) };
        if result == 0 {
            Err(unsafe { winapi::um::errhandlingapi::GetLastError() })
        } else {
            Ok(())
        }
    }

    pub fn prevent_folder_deletion(folder_path: &str) -> Result<(), DWORD> {
        Self::modify_folder_deletion_permissions(folder_path, true)
    }

    pub fn allow_folder_deletion(folder_path: &str) -> Result<(), DWORD> {
        println!("Attempting to allow folder deletion for: {}", folder_path);

        if !Self::verify_password() {
            println!("Password verification failed. Deletion not allowed.");
            return Ok(());
        }

        let wide_path: Vec<u16> = OsStr::new(folder_path)
            .encode_wide()
            .chain(Some(0))
            .collect();

        if let Ok(metadata) = fs::metadata(folder_path) {
            let attributes = metadata.file_attributes();
            if attributes & FILE_ATTRIBUTE_READONLY != 0 {
                let new_attributes = attributes & !FILE_ATTRIBUTE_READONLY;
                let result = unsafe { SetFileAttributesW(wide_path.as_ptr(), new_attributes) };
                if result == 0 {
                    return Err(unsafe { winapi::um::errhandlingapi::GetLastError() });
                }
            }
        }

        if let Err(e) = Self::modify_folder_deletion_permissions(folder_path, false) {
            println!("Failed to modify folder deletion permissions: {:?}", e);
            return Err(e);
        }

        if let Err(e) = Self::remove_folder_attributes(folder_path) {
            println!("Failed to remove folder attributes: {:?}", e);
            return Err(e);
        }

        if let Err(e) = Self::grant_delete_permission_to_everyone(folder_path) {
            println!("Failed to grant delete permission to everyone: {:?}", e);
            return Err(e);
        }

        // Try to rename the folder to itself to release any locks
        let wide_path: Vec<u16> = folder_path.encode_utf16().chain(Some(0)).collect();
        let result = unsafe {
            MoveFileExW(
                wide_path.as_ptr(),
                wide_path.as_ptr(),
                MOVEFILE_REPLACE_EXISTING,
            )
        };
        if result == 0 {
            println!("Failed to release locks on the folder");
        } else {
            println!("Successfully released locks on the folder");
        }

        println!("Successfully allowed folder deletion for: {}", folder_path);
        Ok(())
    }

    fn modify_folder_deletion_permissions(folder_path: &str, deny: bool) -> Result<(), DWORD> {
        // Get the current ACL
        let mut acl = ACL::from_file_path(folder_path, false)?;

        let mut everyone_sid = [0u8; 16];
        let mut sid_size = everyone_sid.len() as u32;
        unsafe {
            CreateWellKnownSid(
                WinWorldSid,
                std::ptr::null_mut(),
                everyone_sid.as_mut_ptr() as PSID,
                &mut sid_size,
            );
        }

        if deny {
            // Deny delete permissions for everyone
            acl.deny(everyone_sid.as_ptr() as *mut _, false, DELETE)?;
            println!("Delete permissions denied for folder: {}", folder_path);
        } else {
            // Remove the deny rule for delete permissions for everyone
            acl.remove(everyone_sid.as_ptr() as *mut _, None, Some(DELETE != 0))?;
            println!("Delete permissions allowed for folder: {}", folder_path);
        }

        Ok(())
    }

    fn grant_delete_permission_to_everyone(folder_path: &str) -> Result<(), DWORD> {
        println!(
            "Granting delete permission to everyone for: {}",
            folder_path
        );

        let mut acl = ACL::from_file_path(folder_path, false)?;

        let mut everyone_sid = [0u8; 16];
        let mut sid_size = everyone_sid.len() as u32;
        unsafe {
            CreateWellKnownSid(
                WinWorldSid,
                std::ptr::null_mut(),
                everyone_sid.as_mut_ptr() as PSID,
                &mut sid_size,
            );
        }

        acl.allow(everyone_sid.as_ptr() as *mut _, false, DELETE)?;
        println!(
            "Delete permission granted to everyone for folder: {}",
            folder_path
        );

        Ok(())
    }

    fn verify_password() -> bool {
        nwg::init().expect("Failed to init Native Windows GUI");
        let mut window = Default::default();
        let mut password = Default::default();
        let mut submit = Default::default();

        nwg::Window::builder()
            .size((300, 115))
            .position((300, 300))
            .title("Enter Password")
            .build(&mut window)
            .expect("Failed to build window");

        nwg::TextInput::builder()
            .text("")
            .position((10, 10))
            .size((280, 25))
            .password(Some('*'))
            .parent(&window)
            .build(&mut password)
            .expect("Failed to build text input");

        nwg::Button::builder()
            .text("Submit")
            .position((100, 45))
            .size((100, 25))
            .parent(&window)
            .build(&mut submit)
            .expect("Failed to build button");

        let window_handle = window.handle;

        let result = Rc::new(RefCell::new(false));
        let result_clone = result.clone();
        let handler =
            nwg::full_bind_event_handler(&window.handle, move |evt, _evt_data, handle| match evt {
                nwg::Event::OnButtonClick => {
                    if handle == submit {
                        if password.text() == "your_password" {
                            *result_clone.borrow_mut() = true;
                        }
                        nwg::stop_thread_dispatch();
                    }
                }
                nwg::Event::OnWindowClose => {
                    if handle == window_handle {
                        nwg::stop_thread_dispatch();
                    }
                }
                _ => {}
            });

        nwg::dispatch_thread_events();
        nwg::unbind_event_handler(&handler);

        Rc::try_unwrap(result).unwrap().into_inner()
    }
}

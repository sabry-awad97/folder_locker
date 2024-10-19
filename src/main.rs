use std::fs;
use std::io::{self, Write};
use std::path::Path;

use winapi::um::fileapi::SetFileAttributesW;
use winapi::um::winnt::{FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_SYSTEM};

const LOCKER_NAME: &str = "Locker";
const HIDDEN_NAME: &str = ".Locker";
const PASSWORD: &str = "123456";

fn main() {
    loop {
        println!("1. Lock Folder");
        println!("2. Unlock Folder");
        println!("3. Exit");
        print!("Choose an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => lock_folder(),
            "2" => unlock_folder(),
            "3" => break,
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn lock_folder() {
    if Path::new(HIDDEN_NAME).exists() {
        println!("Folder is already locked.");
        return;
    }

    if !Path::new(LOCKER_NAME).exists() {
        fs::create_dir(LOCKER_NAME).unwrap();
        println!("Locker folder created.");
    }

    fs::rename(LOCKER_NAME, HIDDEN_NAME).unwrap();
    set_folder_attributes(HIDDEN_NAME);
    println!("Folder locked successfully.");
}

fn unlock_folder() {
    if !Path::new(HIDDEN_NAME).exists() {
        println!("No locked folder found.");
        return;
    }

    print!("Enter password: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim() != PASSWORD {
        println!("Invalid password!");
        return;
    }

    remove_folder_attributes(HIDDEN_NAME);
    fs::rename(HIDDEN_NAME, LOCKER_NAME).unwrap();
    println!("Folder unlocked successfully.");
}

fn set_folder_attributes(name: &str) {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = std::ffi::OsStr::new(name)
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe {
        SetFileAttributesW(
            wide.as_ptr(),
            FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY,
        );
    }
}

fn remove_folder_attributes(name: &str) {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = std::ffi::OsStr::new(name)
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe {
        SetFileAttributesW(wide.as_ptr(), 0);
    }
}

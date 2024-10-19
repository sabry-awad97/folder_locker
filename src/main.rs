use std::fs;
use std::io::Read;
use std::path::Path;

use bcrypt::{hash, verify, DEFAULT_COST};
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Password, Select};
use winapi::um::fileapi::SetFileAttributesW;
use winapi::um::winnt::{FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_SYSTEM};

const LOCKER_NAME: &str = "Locker";
const HIDDEN_NAME: &str = ".Locker";
const METADATA_FILE: &str = ".locker_metadata";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Option<Action>,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Lock,
    Unlock,
}

fn main() {
    let args = Args::parse();

    match args.action {
        Some(Action::Lock) => lock_folder(),
        Some(Action::Unlock) => unlock_folder(),
        None => interactive_mode(),
    }
}

fn interactive_mode() {
    loop {
        let choices = &["Lock Folder", "Unlock Folder", "Exit"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an action")
            .items(choices)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => lock_folder(),
            1 => unlock_folder(),
            2 => break,
            _ => unreachable!(),
        }
        println!();
    }
}

fn lock_folder() {
    if Path::new(HIDDEN_NAME).exists() {
        println!("{}", "Folder is already locked.".yellow());
        return;
    }

    if !Path::new(LOCKER_NAME).exists() {
        fs::create_dir(LOCKER_NAME).unwrap();
        println!("{}", "Locker folder created.".green());
    }

    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a password to lock the folder")
        .with_confirmation("Confirm password", "Passwords don't match")
        .interact()
        .unwrap();

    let hashed_password = hash(password, DEFAULT_COST).unwrap();

    let metadata_path = Path::new(LOCKER_NAME).join(METADATA_FILE);
    fs::write(&metadata_path, &hashed_password).unwrap();
    set_file_attributes(&metadata_path);

    fs::rename(LOCKER_NAME, HIDDEN_NAME).unwrap();
    set_folder_attributes(HIDDEN_NAME);
    println!("{}", "Folder locked successfully.".green().bold());
}

fn unlock_folder() {
    if !Path::new(HIDDEN_NAME).exists() {
        println!("{}", "No locked folder found.".yellow());
        return;
    }

    let input = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter password")
        .interact()
        .unwrap();

    let metadata_path = Path::new(HIDDEN_NAME).join(METADATA_FILE);
    let mut stored_password = String::new();
    fs::File::open(&metadata_path)
        .unwrap()
        .read_to_string(&mut stored_password)
        .unwrap();

    if !verify(input, &stored_password).unwrap() {
        println!("{}", "Invalid password!".red().bold());
        return;
    }

    remove_folder_attributes(HIDDEN_NAME);
    fs::rename(HIDDEN_NAME, LOCKER_NAME).unwrap();
    fs::remove_file(Path::new(LOCKER_NAME).join(METADATA_FILE)).unwrap();
    println!("{}", "Folder unlocked successfully.".green().bold());
}

fn set_file_attributes(path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        SetFileAttributesW(wide.as_ptr(), FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM);
    }
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

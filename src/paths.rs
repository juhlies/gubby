use std::path::PathBuf;
use dirs::home_dir;

pub fn get_gubby_folder() -> PathBuf {
    let user_dir = home_dir().expect("gubby couldn't find user directory");
    user_dir.join(".gubby")
}

pub fn get_gubby_config() -> PathBuf {
    get_gubby_folder().join("gubby.toml")
}

pub fn get_bin_folder() -> PathBuf {
    get_gubby_folder().join("bin")
}

pub fn gubby_installed() -> bool {
    get_gubby_folder().exists()
}

pub fn warn_no_gubby() {
    println!("gubby is not installed, run 'gubby self-install'");
}

pub fn warn_yes_gubby() {
    println!("gubby is already installed, if you're trying to update, run 'gubby self-upgrade'");
}

pub fn warn_no_packages() {
    println!("gubby has no packages installed, run 'gubby install url' to install one");
}
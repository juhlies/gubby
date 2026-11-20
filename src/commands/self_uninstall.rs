use crate::paths::{gubby_installed, get_gubby_folder};
use std::fs;

pub fn run() {
    if !gubby_installed() {println!("gubby already uninstalled."); return;}

    fs::remove_dir_all(get_gubby_folder()).expect("gubby failed to uninstall.");
    
    println!("gubby successfully uninstalled.");
}
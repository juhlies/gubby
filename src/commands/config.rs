use crate::paths::{gubby_installed, warn_no_gubby, get_gubby_config};
use std::process::Command;

pub fn run() {
    if !gubby_installed() {warn_no_gubby(); return;}

    Command::new("cmd")
        .args(["/C", "start", "", &get_gubby_config().to_string_lossy()])
        .spawn()
        .expect("gubby failed to open config");
}
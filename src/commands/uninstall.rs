use crate::paths::{gubby_installed, warn_no_gubby};

pub fn run(url: &str) {
    if !gubby_installed() {warn_no_gubby(); return;}

    println!("gubby uninstalling package {}", url);
}
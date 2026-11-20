use crate::paths::{gubby_installed, warn_no_gubby};

pub fn run(url: Option<&str>) {
    if !gubby_installed() {warn_no_gubby(); return;}

    match url {
        Some(url) => println!("gubby updating package {}", url),
        None => println!("gubby updating all packages"),
    }
}
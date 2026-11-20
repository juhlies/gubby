use crate::paths::{gubby_installed, warn_yes_gubby, get_gubby_folder, get_gubby_config, get_bin_folder};
use serde::{Serialize, Deserialize};
use std::{fs, collections::HashMap};

#[derive(Serialize, Deserialize)]
struct Config {
    packages: HashMap<String, Package>,
}

#[derive(Serialize, Deserialize)]
struct Package {
    version: String,
    url: String,
}

pub fn run() {
    if gubby_installed() {warn_yes_gubby(); return;}
    fs::create_dir_all(get_gubby_folder()).expect("gubby failed to install.");
    fs::create_dir_all(get_bin_folder()).expect("gubby failed to create bin directory.");

    let config = Config {
        packages: HashMap::new(),
    };

    let toml_string = toml::to_string_pretty(&config).expect("gubby failed to serialize config.");
    fs::write(get_gubby_config(), toml_string).expect("gubby failed to create config.");

    println!("gubby successfully installed.");
}
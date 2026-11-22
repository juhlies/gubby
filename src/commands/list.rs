use crate::paths::{get_gubby_config, gubby_installed, warn_no_gubby, warn_no_packages};
use toml_edit::DocumentMut;
use anyhow::{Context, Result};
use std::fs;

pub fn run() -> Result<()> {
    if !gubby_installed() {warn_no_gubby(); return Ok(())}

    let config = fs::read_to_string(get_gubby_config())
        .context("gubby failed to read config")?
        .parse::<DocumentMut>()
        .context("gubby failed to parse config")?;

    let packages = config["packages"]
        .as_table()
        .context("gubby config file is malformed")?;

    if packages.iter().all(|(name, _)| name == "gubby") {
        warn_no_packages();
        return Ok(());
    }

    for (name, package) in packages {
        if name == "gubby" {continue}

        let version = package["version"]
            .as_str()
            .context("gubby package version is missing")?;

        println!("{name} {version}");
    }

    Ok(())
}
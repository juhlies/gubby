use crate::paths::{gubby_installed, warn_no_gubby, get_gubby_config, get_bin_folder};
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::{anyhow, Context, Result};
use crate::github::fetch_repo_path;
use toml_edit::DocumentMut;
use std::fs;

pub fn run(arg: &str) -> Result<()> {
    if !gubby_installed() {warn_no_gubby(); return Ok(())}

    let config_path = get_gubby_config();

    let mut config = fs::read_to_string(&config_path)
        .context("gubby failed to read config")?
        .parse::<DocumentMut>()
        .context("gubby failed to parse config")?;

    let packages = config["packages"]
        .as_table()
        .context("gubby config file is malformed")?;

    let name = if packages.contains_key(arg) {
        arg.to_string()
    } else {
        let repo_path = fetch_repo_path(arg)
            .map_err(|_| anyhow!("gubby received an invalid repository url or unknown package: {arg}"))?;

        packages.iter()
            .find(|(_, package)| {
                package["url"]
                    .as_str()
                    .and_then(|url| fetch_repo_path(url).ok())
                    == Some(repo_path)
            })
            .map(|(name, _)| name.to_string())
            .ok_or_else(|| anyhow!("gubby couldn't find installed package {arg}"))?
    };

    let package = config["packages"]
        .as_table()
        .context("gubby config file is malformed")?
        .get(&name)
        .ok_or_else(|| anyhow!("gubby couldn't find package {name}"))?;

    let binaries = package["binaries"]
        .as_array()
        .context("gubby package binaries are missing")?
        .iter()
        .map(|binary| {
            binary.as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("gubby package binary name is invalid"))
        })
        .collect::<Result<Vec<_>>>()?;

    let total = binaries.len() as u64 + 1;

    let progress = ProgressBar::new(100);
    progress.set_style(
        ProgressStyle::with_template(
            "gubby uninstalling [{bar:30.cyan/blue}] {pos}%: {msg}"
        ).unwrap()
        .progress_chars("=> ")
    );

    progress.set_message(format!("removing {name}"));

    for (index, binary) in binaries.iter().enumerate() {
        fs::remove_file(get_bin_folder().join(binary))
            .with_context(|| format!("gubby failed to remove executable {binary}"))?;

        let percent = ((index as u64 + 1) * 100) / total;
        progress.set_position(percent);
    }

    progress.set_message("updating config");

    config["packages"]
        .as_table_mut()
        .context("gubby config file is malformed")?
        .remove(&name);

    fs::write(&config_path, config.to_string())
        .context("gubby failed to update config")?;

    progress.set_position(100);
    progress.finish_with_message(format!("successfully uninstalled {name}"));

    Ok(())
}
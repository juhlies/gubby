
use crate::paths::{gubby_installed, warn_no_gubby, get_gubby_config, warn_no_packages};
use crate::github::{fetch_repo_path, fetch_releases_url, fetch_releases};
use anyhow::{Context, Result};
use toml_edit::DocumentMut;
use std::fs;

pub async fn run(arg: Option<&str>) -> Result<()> {
    if !gubby_installed() {warn_no_gubby(); return Ok(())}

    let config = fs::read_to_string(get_gubby_config())
        .context("gubby failed to read config")?
        .parse::<DocumentMut>()
        .context("gubby failed to parse config")?;

    let packages = config["packages"]
        .as_table()
        .context("gubby config file is malformed")?;

    let mut installed = Vec::new();

    for (name, package) in packages {
        if name == "gubby" {continue}

        let installed_version = package["version"]
            .as_str()
            .context("gubby package version is missing")?;

        let url = package["url"]
            .as_str()
            .context("gubby package url is missing")?;

        installed.push((name.to_string(), url.to_string(), installed_version.to_string()));
    }

    let packages_to_check = match arg {
        Some(arg) => {
            let package = match installed.iter().find(|(name, _, _)| name == arg) {
                Some(package) => package,
                None => {
                    let repo_path = match fetch_repo_path(arg) {
                        Ok(path) => path,
                        Err(_) => {
                            println!("gubby received an invalid url: {arg}");
                            return Ok(());
                        }
                    };

                    match installed.iter().find(|(_, url, _)| {
                        fetch_repo_path(url).is_ok_and(|path| path == repo_path)
                    }) {
                        Some(package) => package,
                        None => {
                            println!("gubby couldn't find installed package {arg}");
                            return Ok(());
                        }
                    }
                }
            };

            vec![package.clone()]
        }
        None => installed
    };

    if packages_to_check.is_empty() {warn_no_packages(); return Ok(())}

    for (name, url, installed_version) in packages_to_check {
        let releases_url = fetch_releases_url(&url)?;
        let releases = fetch_releases(&releases_url).await?;

        let latest_version = releases.iter()
            .find(|release| !release.draft)
            .map(|release| release.tag_name.strip_prefix("v").unwrap_or(&release.tag_name));

        match latest_version {
            Some(latest) if latest != installed_version => {
                println!("{name} update available: ({installed_version} -> {latest})");
            }
            Some(_) => println!("{name} is up to date ({installed_version})"),
            None => println!("{name} has no releases")
        }
    }

    Ok(())
}
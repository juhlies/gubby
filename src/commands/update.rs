use crate::github::{fetch_repo_path, fetch_releases_url, fetch_releases, latest_stable_release};
use crate::paths::{ get_gubby_config, gubby_installed, warn_no_gubby, warn_no_packages};
use crate::commands::install::install_package;
use anyhow::{Context, Result};
use semver::Version;
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
        if name == "gubby" {
            continue;
        }

        let package_url = package["url"]
            .as_str()
            .context("gubby package url is missing")?;

        let version = package["version"]
            .as_str()
            .context("gubby package version is missing")?;

        installed.push((
            name.to_string(),
            package_url.to_string(),
            version.to_string(),
        ));
    }

    let packages_to_update = match arg {
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
                        fetch_repo_path(url)
                            .is_ok_and(|path| path == repo_path)
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
        None => installed,
    };

    if packages_to_update.is_empty() {
        warn_no_packages();
        return Ok(());
    }

    for (name, url, installed_version) in packages_to_update {
        let releases_url = fetch_releases_url(&url)?;
        let releases = fetch_releases(&releases_url).await?;

        let latest_release = latest_stable_release(&releases);

        match latest_release {
            Some(release) => {
                let latest_version = release
                    .tag_name
                    .strip_prefix('v')
                    .unwrap_or(&release.tag_name);

                let installed_semver = Version::parse(
                    installed_version
                        .strip_prefix('v')
                        .unwrap_or(&installed_version),
                )
                .context("gubby found an invalid installed version")?;

                let latest_semver = Version::parse(latest_version)
                    .context("gubby found an invalid release version")?;

                if latest_semver > installed_semver {
                    install_package(
                        &url,
                        Some(&release.tag_name),
                        "updating",
                        &format!(
                            "updated {name} ({installed_version} -> {latest_version})"
                        ),
                    )
                    .await?;
                } else if arg.is_some() {
                    println!(
                        "gubby {name} is already up to date ({installed_version})"
                    );
                }
            }
            None => {
                println!("gubby found no stable releases for {name}");
            }
        }
    }

    Ok(())
}
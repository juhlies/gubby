use crate::github::{fetch_repo_path, fetch_releases_url, fetch_releases, download, latest_stable_release, find_release, Release};
use crate::paths::{get_bin_folder, get_gubby_config, gubby_installed, warn_no_gubby};
use toml_edit::{DocumentMut, InlineTable, Item, Value, Array};
use archive::{ArchiveExtractor, ArchiveFormat};
use anyhow::{anyhow, Context, Result};
use std::fs;

fn fetch_package_name(url: &str) -> Result<String> {
    url.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .map(String::from)
        .ok_or_else(|| anyhow!("gubby failed to find the package name"))
}

fn is_valid_release(asset_name: &str, _tag_name: &str, _version: Option<&str>) -> bool {
    let name = asset_name.to_lowercase();

    let is_windows = name.contains("windows")
        || name.contains("win64")
        || name.contains("win-x64")
        || name.contains("win_x64");

    let is_other_os = [
        "darwin",
        "macos",
        "linux",
        "ubuntu",
        "freebsd",
        "android",
    ]
    .iter()
    .any(|platform| name.contains(platform));

    let is_arm = [
        "arm64",
        "aarch64",
        "armv7",
    ]
    .iter()
    .any(|arch| name.contains(arch));

    let is_32_bit = name.contains("i386")
        || name.contains("i686")
        || name.contains("win32")
        || (name.contains("x86") && !name.contains("x86_64"));

    is_windows && !is_other_os && !is_arm && !is_32_bit
}

fn fetch_windows_release(releases: &[Release], version: Option<&str>) -> Option<(String, String, String)> {
    let release = match version {
        Some(version) => find_release(releases, version)?,
        None => latest_stable_release(releases)?,
    };

    for asset in &release.assets {
        if is_valid_release(&asset.name, &release.tag_name, version) {
            return Some((
                release.tag_name
                    .strip_prefix("v")
                    .unwrap_or(&release.tag_name)
                    .to_string(),
                asset.browser_download_url.clone(),
                asset.name.clone(),
            ));
        }
    }

    None
}

pub fn fetch_archive_format(file_name: String) -> Option<ArchiveFormat> {
    let name = file_name.to_lowercase();

    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Some(ArchiveFormat::TarGz)
    } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        Some(ArchiveFormat::TarXz)
    } else if name.ends_with(".zip") {
        Some(ArchiveFormat::Zip)
    } else if name.ends_with(".7z") {
        Some(ArchiveFormat::SevenZ)
    } else if name.ends_with(".tar") {
        Some(ArchiveFormat::Tar)
    } else {
        None
    }
}

pub fn extract(archive: &[u8], format: ArchiveFormat) -> Result<Vec<(String, Vec<u8>)>> {
    let files = ArchiveExtractor::new()
        .with_max_file_size(500 * 1024 * 1024)
        .with_max_total_size(1024 * 1024 * 1024)
        .extract(archive, format)
        .context("gubby failed to extract archive")?;

    let mut binaries = Vec::new();

    for file in files {
        if file.is_directory() {continue}

        let path = file.path();
        let name = path.rsplit(['/', '\\']).next().unwrap_or(path);

        if !name.to_lowercase().ends_with(".exe") {continue}

        let data = file
            .data()
            .ok_or_else(|| anyhow!("gubby found an executable, but it contains no data"))?;

        binaries.push((name.to_string(), data.to_vec()));
    }

    if binaries.is_empty() {
        return Err(anyhow!("gubby found no executables in archive"));
    }

    Ok(binaries)
}

pub fn move_binary(name: &str, binary: Vec<u8>) -> Result<()> {
    let destination = get_bin_folder();

    fs::write(destination.join(name), binary)
        .context("gubby failed to write executable")?;

    Ok(())
}

pub fn record_package(name: &str, version: &str, url: &str, binaries: &[String]) -> Result<()> {
    let mut config = fs::read_to_string(get_gubby_config())
        .context("gubby failed to read config")?
        .parse::<DocumentMut>()
        .context("gubby failed to parse config")?;

    let mut package = InlineTable::new();
    package.insert("version", Value::from(version));
    package.insert("url", Value::from(url));

    let mut executables = Array::new();

    for binary in binaries {
        executables.push(binary.as_str());
    }

    package.insert("binaries", Value::Array(executables));

    config["packages"]
        .as_table_mut()
        .context("gubby config file is malformed")?
        .insert(name, Item::Value(Value::InlineTable(package)));

    fs::write(get_gubby_config(), config.to_string())
        .context("gubby failed to update config")?;

    Ok(())
}

pub async fn install_package(url: &str, version: Option<&str>, action: &str, success_action: &str) -> Result<()> {
    let repo_path = fetch_repo_path(url)?;
    let name = fetch_package_name(repo_path)?;
    let releases_url = fetch_releases_url(url)?;
    let releases = fetch_releases(&releases_url).await?;

    let (version, download_url, file_name) = fetch_windows_release(&releases, version)
        .ok_or_else(|| anyhow!("gubby found no valid release for {name}"))?;

    let success_message = if action == "updating" {
        success_action.to_string()
    } else {
        format!("{success_action} {name} ({version})")
    };

    let archive = download(&download_url, action, &success_message).await?;

    let format = fetch_archive_format(file_name)
        .ok_or_else(|| anyhow!("gubby unsupported archive format"))?;

    let binaries = extract(&archive, format)?;

    let mut binary_names = Vec::new();

    for (binary_name, binary) in binaries {
        move_binary(&binary_name, binary)?;
        binary_names.push(binary_name);
    }

    record_package(&name, &version, url, &binary_names)?;

    Ok(())
}

pub async fn run(url: &str, version: Option<&str>, action: &str, success_action: &str) -> Result<()> {
    if !gubby_installed() {warn_no_gubby(); return Ok(())}
    install_package(url, version, action, success_action).await
}
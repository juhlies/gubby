
use crate::paths::{get_bin_folder, gubby_installed, warn_no_gubby};
use crate::github::{fetch_releases_url, fetch_releases, download};
use crate::commands::install::{fetch_archive_format, extract};
use anyhow::{anyhow, Context, Result};
use std::fs;

pub async fn run() -> Result<()> {
    if !gubby_installed() {warn_no_gubby(); return Ok(())}

    let url = "https://github.com/juhlies/gubby.git";
    let releases_url = fetch_releases_url(url)?;
    let releases = fetch_releases(&releases_url).await?;

    let release = releases.iter()
        .find(|release| !release.draft)
        .ok_or_else(|| anyhow!("gubby found no valid releases"))?;

    let asset = release.assets.iter()
        .find(|asset| asset.name.to_lowercase().contains("windows")
        && asset.name.to_lowercase().contains("x86_64"))
        .ok_or_else(|| anyhow!("gubby found no valid release asset"))?;

    let archive = download(&asset.browser_download_url, "updating", &format!("updating to {}", (release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name)))).await?;

    let format = fetch_archive_format(asset.name.clone())
        .ok_or_else(|| anyhow!("gubby unsupported archive format"))?;

    let binaries = extract(&archive, format)?;

    let binary = binaries.into_iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("gubby.exe"))
        .map(|(_, binary)| binary)
        .ok_or_else(|| anyhow!("gubby found no gubby.exe in archive"))?;

    let temporary = get_bin_folder().join("gubby-new.exe");

    fs::write(&temporary, binary)
        .context("gubby failed to write temporary executable")?;

    self_replace::self_replace(&temporary)
        .context("gubby failed to replace executable")?;

    fs::remove_file(&temporary)
        .context("gubby failed to remove temporary executable")?;

    println!("successfully upgraded to {}", release.tag_name);
    Ok(())
}
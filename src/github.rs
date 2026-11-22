use indicatif::{ProgressBar, ProgressStyle};
use anyhow::{Context, Result};
use serde::Deserialize;
use semver::Version;

#[derive(Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn fetch_repo_path(url: &str) -> Result<&str> {
    url.trim_end_matches(".git")
        .trim_end_matches("/")
        .strip_prefix("https://github.com/")
        .context("gubby received an invalid repository url")
}

pub fn fetch_releases_url(url: &str) -> Result<String> {
    Ok(format!(
        "https://api.github.com/repos/{}/releases",
        fetch_repo_path(url)?
    ))
}

pub fn parse_version(version: &str) -> Option<Version> {
    let version = version.strip_prefix('v').unwrap_or(version);

    let (version, prerelease) = match version.split_once('-') {
        Some((version, prerelease)) => (version, Some(prerelease)),
        None => (version, None),
    };

    let mut parts = version.split('.');

    let major = parts.next()?;
    let minor = parts.next().unwrap_or("0");
    let patch = parts.next().unwrap_or("0");

    if parts.next().is_some() {
        return None;
    }

    let version = match prerelease {
        Some(prerelease) => format!("{major}.{minor}.{patch}-{prerelease}"),
        None => format!("{major}.{minor}.{patch}"),
    };

    Version::parse(&version).ok()
}

fn github_request(url: &str) -> reqwest::RequestBuilder {
    let request = reqwest::Client::new()
        .get(url)
        .header("User-Agent", "gubby");

    match std::env::var("GITHUB_TOKEN") {
        Ok(token) => request.bearer_auth(token),
        Err(_) => request,
    }
}

pub async fn fetch_releases(url: &str) -> Result<Vec<Release>> {
    let releases = github_request(url)
        .send()
        .await
        .context("gubby failed to reach github")?
        .error_for_status()
        .context("gubby received a bad response")?
        .json::<Vec<Release>>()
        .await
        .context("gubby failed to parse releases")?;

    Ok(releases)
}

pub fn latest_stable_release(releases: &[Release]) -> Option<&Release> {
    releases
        .iter()
        .filter(|release| !release.draft && !release.prerelease)
        .filter_map(|release| {
            let version = release
                .tag_name
                .strip_prefix('v')
                .unwrap_or(&release.tag_name);

            parse_version(version)
                .map(|version| (release, version))
        })
        .max_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(release, _)| release)
}

pub fn find_release<'a>(
    releases: &'a [Release],
    requested_version: &str,
) -> Option<&'a Release> {
    let requested_version = requested_version
        .strip_prefix('v')
        .unwrap_or(requested_version);

    releases.iter().find(|release| {
        !release.draft
            && release
                .tag_name
                .strip_prefix('v')
                .unwrap_or(&release.tag_name)
                == requested_version
    })
}

pub async fn download(
    url: &str,
    action: &str,
    success_action: &str,
) -> Result<Vec<u8>> {
    let response = github_request(url)
        .send()
        .await
        .context("gubby failed to reach the download server")?
        .error_for_status()
        .context("gubby received a bad download response")?;

    let total_size = response.content_length();

    let progress = match total_size {
        Some(size) if size > 0 => {
            let progress = ProgressBar::new(100);

            progress.set_style(
                ProgressStyle::with_template(
                    &format!(
                        "gubby {action} [{{bar:30.cyan/blue}}] {{pos}}%: {{msg}}"
                    ),
                )
                .unwrap()
                .progress_chars("=> "),
            );

            progress.set_message("");
            progress
        }
        _ => {
            let progress = ProgressBar::new_spinner();

            progress.set_style(
                ProgressStyle::with_template(
                    &format!("gubby {action} {{bytes}}: {{msg}} {{spinner}}"),
                )
                .unwrap(),
            );

            progress.set_message("");
            progress
        }
    };

    let mut bytes = Vec::new();
    let mut downloaded = 0u64;
    let mut response = response;

    while let Some(chunk) = response
        .chunk()
        .await
        .context("gubby failed to download the archive")?
    {
        downloaded += chunk.len() as u64;

        if let Some(size) = total_size.filter(|size| *size > 0) {
            let percent = downloaded.saturating_mul(100) / size;
            progress.set_position(percent.min(100));
        } else {
            progress.inc(chunk.len() as u64);
        }

        bytes.extend_from_slice(&chunk);
    }

    if total_size.is_some_and(|size| size > 0) {
        progress.set_position(100);
    }

    progress.finish_with_message(format!("successfully {success_action}"));

    Ok(bytes)
}
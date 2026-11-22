use crate::paths::{get_bin_folder, get_gubby_config, get_gubby_folder, gubby_installed, warn_yes_gubby};
use crate::commands::install::{install_package, record_package};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::fs::{self, File};

pub async fn run() -> Result<()> {
    if gubby_installed() {warn_yes_gubby(); return Ok(())}

    let url = "https://github.com/juhlies/gubby.git";

    let gubby_folder = get_gubby_folder();
    let bin_folder = get_bin_folder();
    let config_path = get_gubby_config();
    let binary_path = bin_folder.join("gubby.exe");

    let gubby_folder_existed = gubby_folder.exists();
    let bin_folder_existed = bin_folder.exists();

    let old_config = if config_path.exists() {
        Some(fs::read(&config_path).context("gubby failed to back up config")?)
    } else {
        None
    };

    let old_binary = if binary_path.exists() {
        Some(fs::read(&binary_path).context("gubby failed to back up executable")?)
    } else {
        None
    };

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let (environment, _) = current_user
        .create_subkey("Environment")
        .context("gubby failed to access user environment")?;

    let old_path: String = environment.get_value("PATH").unwrap_or_default();
    let mut path_updated = false;

    let result: Result<()> = async {
        fs::create_dir_all(&gubby_folder)
            .context("gubby failed to create directory")?;

        fs::create_dir_all(&bin_folder)
            .context("gubby failed to create bin directory")?;

        if old_config.is_none() {
            fs::write(&config_path, "[packages]\n")
                .context("gubby failed to create config")?;
        }

        if install_package(url, None, "installing", "installed").await.is_err() {
            let executable = std::env::current_exe()
                .context("gubby failed to find current executable")?;

            let mut source = File::open(executable)
                .context("gubby failed to read current executable")?;

            let total_size = source.metadata()
                .context("gubby failed to read executable metadata")?
                .len();

            let total_work = total_size.saturating_mul(2).saturating_add(1);
            let mut completed = 0u64;
            let mut binary = Vec::new();
            let mut buffer = [0u8; 65536];

            let progress = ProgressBar::new(100);
            progress.set_style(
                ProgressStyle::with_template(
                    "gubby installing [{bar:30.cyan/blue}] {pos}%: {msg}"
                ).unwrap()
                .progress_chars("=> ")
            );

            progress.set_message("reading executable");

            loop {
                let count = source.read(&mut buffer)
                    .context("gubby failed to read executable")?;

                if count == 0 {break}

                binary.extend_from_slice(&buffer[..count]);
                completed += count as u64;

                let percent = completed.saturating_mul(100) / total_work;
                progress.set_position(percent.min(99));
            }

            progress.set_message("writing executable");

            let mut destination = File::create(&binary_path)
                .context("gubby failed to create executable")?;

            for chunk in binary.chunks(65536) {
                destination.write_all(chunk)
                    .context("gubby failed to write executable")?;

                completed += chunk.len() as u64;

                let percent = completed.saturating_mul(100) / total_work;
                progress.set_position(percent.min(99));
            }

            progress.set_message("registering package");

            record_package("gubby", env!("CARGO_PKG_VERSION"), url, &["gubby.exe".to_string()])?;

            progress.set_position(100);
            progress.finish_with_message("successfully self installed");
        }

        let bin = bin_folder.to_string_lossy().to_string();

        if !old_path.split(';').any(|p| p.eq_ignore_ascii_case(&bin)) {
            let mut path = old_path.clone();

            if !path.is_empty() {path.push(';')}
            path.push_str(&bin);

            environment.set_value("PATH", &path)
                .context("gubby failed to update PATH")?;

            path_updated = true;
        }

        Ok(())
    }.await;

    if let Err(error) = result {
        if path_updated {
            let _ = environment.set_value("PATH", &old_path);
        }

        match old_config {
            Some(config) => {let _ = fs::write(&config_path, config);}
            None => {let _ = fs::remove_file(&config_path);}
        }

        match old_binary {
            Some(binary) => {let _ = fs::write(&binary_path, binary);}
            None => {let _ = fs::remove_file(&binary_path);}
        }

        if !gubby_folder_existed {
            let _ = fs::remove_dir_all(&gubby_folder);
        } else if !bin_folder_existed {
            let _ = fs::remove_dir(&bin_folder);
        }

        return Err(error);
    }

    Ok(())
}
use crate::paths::{gubby_installed, get_gubby_folder};
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

fn collect_paths(path: &Path, entries: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)
        .context("gubby failed to read installation directory")? {
        let entry = entry.context("gubby failed to read directory entry")?;
        let entry_path = entry.path();

        if entry.file_type()
            .context("gubby failed to read file type")?.is_dir() {
            collect_paths(&entry_path, entries)?;
        } else {
            entries.push(entry_path);
        }
    }

    entries.push(path.to_path_buf());

    Ok(())
}

pub fn run() -> Result<()> {
    if !gubby_installed() {println!("gubby already uninstalled"); return Ok(())}

    let folder = get_gubby_folder();

    self_replace::self_delete_outside_path(&folder)
        .context("gubby failed to schedule executable deletion")?;

    let mut entries = Vec::new();
    collect_paths(&folder, &mut entries)?;

    let total = entries.len() as u64;

    let progress = ProgressBar::new(100);
    progress.set_style(
        ProgressStyle::with_template(
            "gubby uninstalling [{bar:30.cyan/blue}] {pos}%: {msg}"
        ).unwrap()
        .progress_chars("=> ")
    );

    progress.set_message("removing files");

    for (index, path) in entries.iter().enumerate() {
        if path.is_dir() {
            fs::remove_dir(path)
                .context("gubby failed to remove directory")?;
        } else {
            fs::remove_file(path)
                .context("gubby failed to remove file")?;
        }

        let percent = ((index as u64 + 1) * 100) / total;
        progress.set_position(percent);
    }

    progress.finish_with_message("successfully uninstalled");

    Ok(())
}
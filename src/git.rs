use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn dest_exists(dest: &str) -> bool {
    std::path::Path::new(dest).exists()
}

pub fn clone_repo(url: &str, dest: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(dest)
        .status()
        .context("failed to run git clone")?;

    if !status.success() {
        bail!("git clone failed for {url}");
    }

    Ok(())
}

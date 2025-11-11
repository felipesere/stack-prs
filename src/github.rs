use anyhow::{Context, Result};
use std::process::Command;
use log::debug;

/// Create a pull request using the GitHub CLI (gh)
pub fn create_pr(head_branch: &str, base_branch: &str, title: &str) -> Result<()> {
    debug!(
        "Executing command: gh pr create --head {} --base {} --title {} --body \"\"",
        head_branch, base_branch, title
    );

    let output = Command::new("gh")
        .arg("pr")
        .arg("create")
        .arg("--head")
        .arg(head_branch)
        .arg("--base")
        .arg(base_branch)
        .arg("--title")
        .arg(title)
        .arg("--body")
        .arg("") // Empty body for now
        .output()
        .context("Failed to execute gh pr create. Make sure GitHub CLI (gh) is installed and authenticated.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr create failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("PR created: {}", stdout.trim());

    Ok(())
}

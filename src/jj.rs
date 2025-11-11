use anyhow::{Context, Result};
use std::process::Command;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Change {
    pub change_id: String,
    pub description: String,
    pub bookmark: Option<String>,
}

/// Get all changes between base and target that are mine()
pub fn get_changes(base: &str, target: &str) -> Result<Vec<Change>> {
    let revisions_arg = format!("{base}..{target} & mine()");
    let template_arg = "change_id ++ \"\\n\" ++ description.trim() ++ \"\\n\" ++ local_bookmarks.join(\",\") ++ \"\\n---\\n\"";

    debug!(
        command = "jj",
        args = ?["log", "--no-graph", "--revisions", &revisions_arg, "--template", template_arg],
        "Executing command"
    );

    let output = Command::new("jj")
        .arg("log")
        .arg("--no-graph")
        .arg("--revisions")
        .arg(&revisions_arg)
        .arg("--template")
        .arg(template_arg)
        .output()
        .context("Failed to execute jj log command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("jj log failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_jj_log(&stdout)
}

fn parse_jj_log(output: &str) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    let entries: Vec<&str> = output.split("---\n").collect();

    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        let lines: Vec<&str> = entry.lines().collect();
        if lines.is_empty() {
            continue;
        }

        let change_id = lines[0].trim().to_string();
        let description = if lines.len() > 1 {
            lines[1].trim().to_string()
        } else {
            String::new()
        };

        let bookmark = if lines.len() > 2 && !lines[2].trim().is_empty() {
            Some(lines[2].trim().to_string())
        } else {
            None
        };

        changes.push(Change {
            change_id,
            description,
            bookmark,
        });
    }

    Ok(changes)
}

/// Create a bookmark for a given change
pub fn create_bookmark(change_id: &str, bookmark_name: &str) -> Result<()> {
    debug!(
        command = "jj",
        args = ?["bookmark", "create", bookmark_name, "--revision", change_id],
        "Executing command"
    );

    let output = Command::new("jj")
        .arg("bookmark")
        .arg("create")
        .arg(bookmark_name)
        .arg("--revision")
        .arg(change_id)
        .output()
        .context("Failed to execute jj bookmark create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("jj bookmark create failed: {stderr}");
    }

    Ok(())
}

/// Push a bookmark to the remote
pub fn push_bookmark(bookmark_name: &str) -> Result<()> {
    debug!(
        command = "jj",
        args = ?["git", "push", "--bookmark", bookmark_name, "--allow-new"],
        "Executing command"
    );

    let output = Command::new("jj")
        .arg("git")
        .arg("push")
        .arg("--bookmark")
        .arg(bookmark_name)
        .arg("--allow-new")
        .output()
        .context("Failed to execute jj git push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("jj git push failed: {stderr}");
    }

    Ok(())
}

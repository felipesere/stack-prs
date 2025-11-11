use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;
use tracing::debug;

use crate::jj::Change;

#[derive(Debug, Clone)]
pub enum Action {
    Skip,
    CreateBookmark,
    CreatePr,
}

#[derive(Debug, Clone)]
pub struct StackEntry {
    pub action: Action,
    pub change_id: String,
    pub description: String,
    pub bookmark: Option<String>,
}

const HEADER: &str = r#"# The following file represents your stack in the order it will applied, top to bottom.
# The first column can be one of:
# * "skip" or "s": to skip this change entirely (can also just delete the line)
# * "create-pr" or "pr": to create the PR based on an already existing bookmark
# * "bookmark" or "b": to create a named bookmark to then use for the PR
# the other columns are:
# * the change ID
# * the change description
# * if present, the bookmark
"#;

/// Create a temporary file with the stack, open it in $EDITOR, and parse the result
pub fn edit_stack(changes: Vec<Change>) -> Result<Vec<StackEntry>> {
    // Create the initial stack file content
    let mut content = String::from(HEADER);

    for change in changes.iter().rev() {
        let action = if change.bookmark.is_some() {
            "pr"
        } else {
            "bookmark"
        };

        let bookmark_str = change.bookmark.as_deref().unwrap_or("");
        content.push_str(&format!(
            "{},{},{},{}\n",
            action, change.change_id, change.description, bookmark_str
        ));
    }

    // Create a temporary file
    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;

    temp_file
        .write_all(content.as_bytes())
        .context("Failed to write to temporary file")?;

    let temp_path = temp_file.path().to_owned();

    // Get the editor from environment
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    debug!(
        command = %editor,
        args = ?[&temp_path.to_string_lossy().to_string()],
        "Executing command"
    );

    // Open the editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .context(format!("Failed to open editor: {editor}"))?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    // Read back the edited file
    let edited_content = fs::read_to_string(&temp_path).context("Failed to read edited file")?;

    // Parse the edited content
    parse_stack_file(&edited_content)
}

fn parse_stack_file(content: &str) -> Result<Vec<StackEntry>> {
    let mut entries = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 {
            continue; // Skip malformed lines
        }

        let action_str = parts[0].trim();
        let change_id = parts[1].trim().to_string();
        let description = parts[2].trim().to_string();
        let bookmark = if parts.len() > 3 && !parts[3].trim().is_empty() {
            Some(parts[3].trim().to_string())
        } else {
            None
        };

        let action = match action_str {
            "skip" | "s" => Action::Skip,
            "bookmark" | "b" => Action::CreateBookmark,
            "create-pr" | "pr" => Action::CreatePr,
            _ => {
                eprintln!("Warning: Unknown action '{action_str}', skipping line");
                continue;
            }
        };

        entries.push(StackEntry {
            action,
            change_id,
            description,
            bookmark,
        });
    }

    Ok(entries)
}

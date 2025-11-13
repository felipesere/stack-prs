use anyhow::{Context, Result};
use std::process::Command;
use log::debug;

/// Check if a PR exists for a given branch
pub fn pr_exists(branch: &str) -> Result<bool> {
    debug!(
        "Executing command: gh pr list --head {} --json number",
        branch
    );

    let output = Command::new("gh")
        .arg("pr")
        .arg("list")
        .arg("--head")
        .arg(branch)
        .arg("--json")
        .arg("number")
        .output()
        .context("Failed to execute gh pr list. Make sure GitHub CLI (gh) is installed and authenticated.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr list failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // If the output is "[]", no PRs exist
    Ok(!stdout.trim().is_empty() && stdout.trim() != "[]")
}

/// Create a pull request using the GitHub CLI (gh) and return the PR URL
pub fn create_pr(head_branch: &str, base_branch: &str, title: &str) -> Result<String> {
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
    let pr_url = stdout.trim().to_string();
    println!("PR created: {}", pr_url);

    Ok(pr_url)
}

/// Add a comment to a PR using the PR URL
pub fn add_pr_comment(pr_url: &str, comment: &str) -> Result<()> {
    debug!(
        "Executing command: gh pr comment {} --body \"{}\"",
        pr_url, comment
    );

    let output = Command::new("gh")
        .arg("pr")
        .arg("comment")
        .arg(pr_url)
        .arg("--body")
        .arg(comment)
        .output()
        .context("Failed to execute gh pr comment")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr comment failed: {stderr}");
    }

    Ok(())
}

/// Get the PR URL and title for an existing PR by branch name
pub fn get_pr_info(branch: &str) -> Result<(String, String)> {
    debug!(
        "Executing command: gh pr list --head {} --json url,title",
        branch
    );

    let output = Command::new("gh")
        .arg("pr")
        .arg("list")
        .arg("--head")
        .arg(branch)
        .arg("--json")
        .arg("url,title")
        .output()
        .context("Failed to execute gh pr list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr list failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON to extract url and title
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .context("Failed to parse gh pr list JSON output")?;

    let pr_array = json.as_array()
        .context("Expected JSON array from gh pr list")?;

    if pr_array.is_empty() {
        anyhow::bail!("No PR found for branch {}", branch);
    }

    let pr_obj = &pr_array[0];
    let pr_url = pr_obj["url"].as_str()
        .context("Missing 'url' field in PR JSON")?
        .to_string();
    let pr_title = pr_obj["title"].as_str()
        .context("Missing 'title' field in PR JSON")?
        .to_string();

    Ok((pr_url, pr_title))
}

/// Check if a stack comment already exists on a PR and return its ID if found
fn get_stack_comment_id(pr_url: &str) -> Result<Option<String>> {
    debug!(
        "Executing command: gh pr view {} --json comments --jq '.comments[] | select(.body | contains(\"## Stack Information\")) | .id'",
        pr_url
    );

    let output = Command::new("gh")
        .arg("pr")
        .arg("view")
        .arg(pr_url)
        .arg("--json")
        .arg("comments")
        .arg("--jq")
        .arg(".comments[] | select(.body | contains(\"## Stack Information\")) | .id")
        .output()
        .context("Failed to execute gh pr view")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr view failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let comment_id = stdout.trim();

    if comment_id.is_empty() {
        Ok(None)
    } else {
        // If multiple comments exist, take the first one
        Ok(Some(comment_id.lines().next().unwrap_or("").to_string()))
    }
}

/// Update an existing comment on a PR
fn update_pr_comment(pr_url: &str, comment_id: &str, comment: &str) -> Result<()> {
    debug!(
        "Executing command: gh api -X PATCH /repos/{{owner}}/{{repo}}/issues/comments/{} -f body=...",
        comment_id
    );

    // Extract owner/repo from PR URL
    // PR URL format: https://github.com/owner/repo/pull/123
    let parts: Vec<&str> = pr_url.trim_end_matches('/').split('/').collect();
    if parts.len() < 5 {
        anyhow::bail!("Invalid PR URL format: {}", pr_url);
    }
    let owner = parts[parts.len() - 4];
    let repo = parts[parts.len() - 3];

    let api_endpoint = format!("/repos/{}/{}/issues/comments/{}", owner, repo, comment_id);

    let output = Command::new("gh")
        .arg("api")
        .arg("-X")
        .arg("PATCH")
        .arg(&api_endpoint)
        .arg("-f")
        .arg(format!("body={}", comment))
        .output()
        .context("Failed to execute gh api")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh api failed: {stderr}");
    }

    Ok(())
}

/// Add or update a stack comment on a PR
pub fn add_or_update_stack_comment(pr_url: &str, comment: &str) -> Result<()> {
    if let Some(comment_id) = get_stack_comment_id(pr_url)? {
        debug!("Updating existing stack comment {} on PR {}", comment_id, pr_url);
        update_pr_comment(pr_url, &comment_id, comment)?;
    } else {
        debug!("Adding new stack comment to PR {}", pr_url);
        add_pr_comment(pr_url, comment)?;
    }
    Ok(())
}

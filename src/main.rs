mod jj;
mod stack;
mod github;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "stack-prs")]
#[command(about = "Create stacked PRs on GitHub using jj", long_about = None)]
struct Args {
    /// Base revision (defaults to trunk())
    #[arg(long, default_value = "trunk()")]
    base: String,

    /// Target revision (defaults to @)
    #[arg(long, default_value = "@")]
    target: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Get all changes between base and target that are mine()
    let changes = jj::get_changes(&args.base, &args.target)?;

    // Create and edit the stack file
    let stack_entries = stack::edit_stack(changes)?;

    // Process the stack entries
    process_stack(stack_entries)?;

    Ok(())
}

fn process_stack(entries: Vec<stack::StackEntry>) -> Result<()> {
    let mut previous_branch: Option<String> = None;

    for entry in entries {
        match entry.action {
            stack::Action::Skip => {
                println!("Skipping change {}", entry.change_id);
                continue;
            }
            stack::Action::CreateBookmark => {
                let bookmark = entry.bookmark.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Bookmark name required for action 'bookmark'"))?;

                println!("Creating bookmark '{}' for change {}", bookmark, entry.change_id);
                jj::create_bookmark(&entry.change_id, bookmark)?;
                jj::push_bookmark(bookmark)?;

                let base_branch = previous_branch.as_deref().unwrap_or("main");
                println!("Creating PR for '{bookmark}' against '{base_branch}'");
                github::create_pr(bookmark, base_branch, &entry.description)?;

                previous_branch = Some(bookmark.clone());
            }
            stack::Action::CreatePr => {
                let bookmark = entry.bookmark.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Bookmark name required for action 'pr'"))?;

                let base_branch = previous_branch.as_deref().unwrap_or("main");
                println!("Creating PR for existing bookmark '{bookmark}' against '{base_branch}'");
                github::create_pr(bookmark, base_branch, &entry.description)?;

                previous_branch = Some(bookmark.clone());
            }
        }
    }

    Ok(())
}

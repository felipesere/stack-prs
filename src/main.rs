mod github;
mod jj;
mod stack;

use anyhow::Result;
use bpaf::*;
use owo_colors::OwoColorize;

#[derive(Debug, Clone)]
struct Args {
    revisions: String,
    verbose: usize,
}

fn args() -> OptionParser<Args> {
    let revisions = long("revisions")
        .short('r')
        .help("Revision to consider for stack. Defaults to trunk()::@")
        .argument::<String>("REVISION")
        .fallback("trunk()::@".to_string());

    let verbose = short('v')
        .long("verbose")
        .help("Increase the verbosity\n You can specify it up to 3 times\n either as -v -v -v or as -vvv")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this");

    construct!(Args { revisions, verbose })
        .to_options()
        .descr("Create stacked PRs on GitHub using jj")
}

fn main() -> Result<()> {
    let args = args().run();

    setup_logging(args.verbose)?;

    // Get all changes between base and target that are mine()
    let changes = jj::get_changes(&args.revisions)?;

    // Create and edit the stack file
    let stack_entries = stack::edit_stack(changes)?;

    // Process the stack entries
    process_stack(stack_entries)?;

    Ok(())
}

fn setup_logging(verbosity: usize) -> Result<(), anyhow::Error> {
    let mut base_config = fern::Dispatch::new().format(move |out, message, record| {
        let level = match record.level() {
            log::Level::Error => "ERROR".red().to_string(),
            log::Level::Warn => "WARN".yellow().to_string(),
            log::Level::Info => "INFO".blue().to_string(),
            log::Level::Debug => "DEBUG".green().to_string(),
            log::Level::Trace => "TRACE".magenta().to_string(),
        };

        let module = record.module_path().unwrap_or("unknown");

        out.finish(format_args!("{level}:{module}: {message}",))
    });

    base_config = match verbosity {
        0 => base_config.level(log::LevelFilter::Warn),
        1 => base_config
            .level(log::LevelFilter::Debug)
            .level_for("rustls", log::LevelFilter::Warn),
        2 => base_config.level(log::LevelFilter::Debug),
        3 => base_config.level(log::LevelFilter::Trace),
        _ => unreachable!("verbosity > 3"),
    };
    base_config.chain(std::io::stderr()).apply()?;

    Ok(())
}

struct ProcessedPr {
    pr_url: String,
    pr_title: String,
}

fn process_stack(entries: Vec<stack::StackEntry>) -> Result<()> {
    let mut previous_branch: Option<String> = None;
    let mut processed_prs: Vec<ProcessedPr> = Vec::new();

    // Count total PRs to be created (excluding skips)
    let total_prs = entries
        .iter()
        .filter(|e| matches!(e.action, stack::Action::CreatePr))
        .count();

    // First pass: Create/collect all PRs
    for entry in entries {
        match entry.action {
            stack::Action::Skip => {
                println!("Skipping change {}", entry.change_id);
                continue;
            }
            stack::Action::CreatePr => {
                let base_branch = previous_branch.as_deref().unwrap_or("main");

                // Determine which bookmark to use and get PR URL + title
                let (bookmark, pr_url, pr_title) = if let Some(bookmark_name) =
                    entry.bookmark.as_ref()
                {
                    // User provided a bookmark name (either existing or new)
                    // Check if PR already exists for this bookmark
                    if github::pr_exists(bookmark_name)? {
                        println!(
                            "PR already exists for bookmark '{bookmark_name}', keeping in stack"
                        );
                        let (pr_url, pr_title) = github::get_pr_info(bookmark_name)?;
                        (bookmark_name.clone(), pr_url, pr_title)
                    } else {
                        // Need to create PR - bookmark might already exist or need to be created
                        // Try to push the bookmark first, which will work if it exists
                        // If it doesn't exist, create it first
                        match jj::push_bookmark(bookmark_name) {
                            Ok(_) => {
                                println!("Creating PR for bookmark '{bookmark_name}' against '{base_branch}'");
                                let pr_url = github::create_pr(
                                    bookmark_name,
                                    base_branch,
                                    &entry.description,
                                )?;
                                (bookmark_name.clone(), pr_url, entry.description.clone())
                            }
                            Err(_) => {
                                // Bookmark doesn't exist, create it
                                println!(
                                    "Creating bookmark '{bookmark_name}' for change {}",
                                    entry.change_id
                                );
                                jj::create_bookmark(&entry.change_id, bookmark_name)?;
                                jj::push_bookmark(bookmark_name)?;
                                println!("Creating PR for bookmark '{bookmark_name}' against '{base_branch}'");
                                let pr_url = github::create_pr(
                                    bookmark_name,
                                    base_branch,
                                    &entry.description,
                                )?;
                                (bookmark_name.clone(), pr_url, entry.description.clone())
                            }
                        }
                    }
                } else {
                    // No bookmark provided, let jj create an automatic one
                    println!(
                        "No bookmark for change {}, creating automatic bookmark",
                        entry.change_id
                    );
                    let auto_bookmark = jj::push_change_auto_bookmark(&entry.change_id)?;
                    println!("Created automatic bookmark '{auto_bookmark}', creating PR against '{base_branch}'");
                    let pr_url =
                        github::create_pr(&auto_bookmark, base_branch, &entry.description)?;
                    (auto_bookmark, pr_url, entry.description.clone())
                };

                processed_prs.push(ProcessedPr { pr_url, pr_title });
                previous_branch = Some(bookmark);
            }
        }
    }

    // Second pass: Add stack comments to all PRs
    for (index, pr) in processed_prs.iter().enumerate() {
        let position = index + 1;
        let mut comment = format!(
            "## Stack Information\n\nThis PR is **{} of {}** in the stack.\n",
            position, total_prs
        );

        if index > 0 {
            let prev_pr = &processed_prs[index - 1];
            comment.push_str(&format!(
                "\n⬇️ Previous PR: [{}]({})\n",
                prev_pr.pr_title, prev_pr.pr_url
            ));
        }

        if index < processed_prs.len() - 1 {
            let next_pr = &processed_prs[index + 1];
            comment.push_str(&format!(
                "\n⬆️ Next PR: [{}]({})\n",
                next_pr.pr_title, next_pr.pr_url
            ));
        }

        println!("Adding/updating stack comment on PR: {}", pr.pr_url);
        github::add_or_update_stack_comment(&pr.pr_url, &comment)?;
    }

    Ok(())
}

mod github;
mod jj;
mod stack;

use anyhow::Result;
use bpaf::*;
use owo_colors::OwoColorize;

#[derive(Debug, Clone)]
struct Args {
    base: String,
    target: String,
    verbose: usize,
}

fn args() -> OptionParser<Args> {
    let base = long("base")
        .help("Base revision (defaults to trunk())")
        .argument::<String>("REVISION")
        .fallback(String::from("trunk()"));

    let target = long("target")
        .help("Target revision (defaults to @)")
        .argument::<String>("REVISION")
        .fallback(String::from("@"));

    let verbose = short('v')
        .long("verbose")
        .help("Increase the verbosity\n You can specify it up to 3 times\n either as -v -v -v or as -vvv")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this");

    construct!(Args {
        base,
        target,
        verbose
    })
    .to_options()
    .descr("Create stacked PRs on GitHub using jj")
}

fn main() -> Result<()> {
    let args = args().run();

    setup_logging(args.verbose)?;

    // Get all changes between base and target that are mine()
    let changes = jj::get_changes(&args.base, &args.target)?;

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

fn process_stack(entries: Vec<stack::StackEntry>) -> Result<()> {
    let mut previous_branch: Option<String> = None;

    for entry in entries {
        match entry.action {
            stack::Action::Skip => {
                println!("Skipping change {}", entry.change_id);
                continue;
            }
            stack::Action::CreateBookmark => {
                let bookmark = entry.bookmark.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("Bookmark name required for action 'bookmark'")
                })?;

                println!(
                    "Creating bookmark '{}' for change {}",
                    bookmark, entry.change_id
                );
                jj::create_bookmark(&entry.change_id, bookmark)?;
                jj::push_bookmark(bookmark)?;

                let base_branch = previous_branch.as_deref().unwrap_or("main");
                println!("Creating PR for '{bookmark}' against '{base_branch}'");
                github::create_pr(bookmark, base_branch, &entry.description)?;

                previous_branch = Some(bookmark.clone());
            }
            stack::Action::CreatePr => {
                let bookmark = entry
                    .bookmark
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Bookmark name required for action 'pr'"))?;

                let base_branch = previous_branch.as_deref().unwrap_or("main");
                jj::push_bookmark(bookmark)?;
                println!("Creating PR for existing bookmark '{bookmark}' against '{base_branch}'");
                github::create_pr(bookmark, base_branch, &entry.description)?;

                previous_branch = Some(bookmark.clone());
            }
        }
    }

    Ok(())
}

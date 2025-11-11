# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`stack-prs` is a Rust CLI tool that creates stacked pull requests on GitHub using Jujutsu (`jj`) version control. It enables managing dependent PRs in a stack where each PR builds upon the previous one.

## Development Commands

### Build and Run
```bash
cargo build                    # Build the project
cargo build --release          # Build optimized release version
cargo run                      # Run with default arguments (trunk()..@ & mine())
cargo run -- --base trunk() --target @   # Run with explicit arguments
```

### Testing
```bash
cargo test                     # Run all tests
cargo test <test_name>         # Run a specific test
RUST_LOG=debug cargo run       # Run with debug logging
```

### Debugging
Use the `RUST_LOG` environment variable to control log levels:
```bash
RUST_LOG=debug cargo run       # Debug level logging
RUST_LOG=trace cargo run       # Trace level logging
RUST_LOG=info cargo run        # Info level logging (default)
```

## Architecture

### High-Level Flow
1. **Query Changes** (`jj` module): Fetches changes between two revisions using `jj log`
2. **Interactive Editing** (`stack` module): Opens an editor with a CSV-like file for the user to specify actions
3. **Process Stack** (`main.rs`): Iterates through entries, creating bookmarks and PRs in sequence
4. **GitHub Integration** (`github` module): Creates PRs using the `gh` CLI

### Module Responsibilities

**`jj.rs`** - Jujutsu Integration
- Queries changes using `jj log` with custom templates
- Parses change IDs, descriptions, and bookmarks
- Creates bookmarks for specific revisions
- Pushes bookmarks to remote with `jj git push`

**`stack.rs`** - Stack File Editor
- Generates a CSV-formatted file with header instructions
- Opens the user's `$EDITOR` (defaults to `vi`) for editing
- Parses the edited file to extract actions:
  - `skip`/`s`: Skip this change
  - `bookmark`/`b`: Create a bookmark and PR
  - `create-pr`/`pr`: Use existing bookmark to create PR
- Returns a list of `StackEntry` structs with actions to perform

**`github.rs`** - GitHub Integration
- Creates PRs using `gh pr create`
- Requires GitHub CLI (`gh`) to be installed and authenticated

**`main.rs`** - Orchestration
- Parses CLI arguments (base and target revisions)
- Coordinates the workflow: fetch changes → edit stack → process entries
- Maintains "previous branch" state to create stacked PRs where each PR targets the previous branch
- First PR targets `main`, subsequent PRs target the previous PR's branch

### Key Design Patterns

**Stacking Logic**: The tool processes entries top-to-bottom, maintaining `previous_branch` state. Each PR is created against the previous PR's branch, creating a dependency chain. The first PR always targets `main`.

**Error Handling**: Uses `anyhow::Result` throughout for error propagation with context. Command failures include stderr output in error messages.

**Logging**: Structured logging with `tracing` crate. Commands are logged at debug level before execution.

## Prerequisites

This tool requires:
- `jj` (Jujutsu) - for version control operations
- `gh` (GitHub CLI) - for creating pull requests, must be authenticated
- Editor configured via `$EDITOR` environment variable

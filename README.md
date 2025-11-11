# stack-prs

A powerful CLI tool for creating stacked pull requests on GitHub using [Jujutsu](https://github.com/martinvonz/jj) version control.

## What are Stacked PRs?

Stacked PRs allow you to work on multiple dependent changes simultaneously. Instead of waiting for one PR to be reviewed and merged before starting the next, you can:

- Build features incrementally with each PR building on the previous one
- Get faster feedback on individual changes
- Maintain a clear dependency chain where PR 2 depends on PR 1, PR 3 depends on PR 2, etc.

## Features

- 📝 **Interactive editing** - Review and organize your changes in your favorite editor
- 🔗 **Automatic stacking** - Each PR automatically targets the previous PR's branch
- 🎯 **Flexible actions** - Create new bookmarks, use existing ones, or skip changes

## Prerequisites

Before using `stack-prs`, ensure you have:

1. **[Jujutsu (jj)](https://github.com/martinvonz/jj)** - Version control system
2. **[GitHub CLI (gh)](https://cli.github.com/)** - Must be authenticated (`gh auth login`)
3. **Editor** - Set via `$EDITOR` environment variable (defaults to `vi`)

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

### Basic Usage

Run with defaults (trunk()..@ & mine()):

```bash
stack-prs
```

### Custom Revisions

Specify custom base and target revisions:

```bash
stack-prs --base main --target @
stack-prs --base trunk() --target my-feature
```

## How It Works

### 1. Query Changes

`stack-prs` fetches all your changes between the base and target revisions:

```bash
stack-prs --base trunk() --target @
```

### 2. Interactive Editing

Your `$EDITOR` opens with a CSV-formatted file showing your changes:

```csv
# The following file represents your stack in the order it will applied, top to bottom.
# The first column can be one of:
# * "skip" or "s": to skip this change entirely (can also just delete the line)
# * "create-pr" or "pr": to create the PR based on an already existing bookmark
# * "bookmark" or "b": to create a named bookmark to then use for the PR
# the other columns are:
# * the change ID
# * the change description
# * if present, the bookmark

bookmark,pzkkouuwrxkrpoxqknztyqkpwtuqzqmz,Pass the architecture down to the Helm chart on render,enops-2222
bookmark,utounnzrstvosknnorusyryvwywwqlwp,Detect arch with uname,enops-1111
pr,rzpwqyytylqxowwlmywkpvpyqwlzuzyy,Create multi arch image,enops-1234
s,nsqzmntuqwqulqnxnwnxkypqtqklstov,Use alpha releaser to release releaser,
s,tvqnnqqmvtmsqsvwootxswqvrowwxnrs,Empty commit to re-trigger CI,arm-detection
```

### 3. Define Actions

Edit the file to specify what to do with each change:

| Action | Aliases | Description | Requires Bookmark |
|--------|---------|-------------|-------------------|
| `bookmark` | `b` | Create a new bookmark and PR | ✅ Yes (4th column) |
| `create-pr` | `pr` | Use existing bookmark to create PR | ✅ Yes (already exists) |
| `skip` | `s` | Skip this change | ❌ No |

### 4. Automatic Stacking

When you save and close the editor, `stack-prs` processes your changes **top-to-bottom**:

1. **First PR** → targets `main` (or your default branch)
2. **Second PR** → targets the first PR's branch
3. **Third PR** → targets the second PR's branch
4. And so on...

## Example Workflow

Given this edited file:

```csv
bookmark,abc123,Add user authentication,feature/auth
bookmark,def456,Add user profile page,feature/profile
bookmark,ghi789,Add profile settings,feature/settings
s,jkl012,WIP: debugging,
```

`stack-prs` will:

1. ✅ Create bookmark `feature/auth` for change `abc123`
   - Push to GitHub
   - Create PR: `feature/auth` → `main`

2. ✅ Create bookmark `feature/profile` for change `def456`
   - Push to GitHub
   - Create PR: `feature/profile` → `feature/auth` (stacked!)

3. ✅ Create bookmark `feature/settings` for change `ghi789`
   - Push to GitHub
   - Create PR: `feature/settings` → `feature/profile` (stacked!)

4. ⏭️ Skip change `jkl012`

The result is a chain of dependent PRs:
```
main ← feature/auth ← feature/profile ← feature/settings
```

## Tips

- **Review before running** - The interactive editor lets you review all changes before creating PRs
- **Use descriptive branch names** - They become your PR titles and make the stack easy to navigate
- **Reorder changes** - Edit the file to change the order of PRs in your stack

## Architecture

The codebase is organized into focused modules:

- **`main.rs`** - CLI parsing (bpaf), logging (fern), and orchestration
- **`jj.rs`** - Jujutsu integration (log, bookmark, push)
- **`github.rs`** - GitHub CLI integration (create PRs)
- **`stack.rs`** - Interactive editor and CSV parsing

## Contributing

Contributions are welcome! Please ensure:

- Code builds with `cargo build`
- Follow existing code style
- Update documentation for new features

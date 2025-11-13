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
- 🎯 **Flexible bookmarks** - Add custom bookmark names or let jj auto-generate them
- 🔄 **Smart PR handling** - Detects existing PRs and keeps them in the stack
- 💬 **Stack navigation comments** - Automatically adds comments to PRs with links to previous/next PRs in the stack

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
# * "pr": to create or update a PR for this change
#         If a bookmark exists and has a PR, it will be kept in the stack
#         If a bookmark exists without a PR, a PR will be created
#         If no bookmark exists, jj will create an automatic one
# the other columns are:
# * the change ID
# * the change description
# * if present, the bookmark name (can be added/edited if not set)

pr,pzkkouuwrxkrpoxqknztyqkpwtuqzqmz,Pass the architecture down to the Helm chart on render,enops-2222
pr,utounnzrstvosknnorusyryvwywwqlwp,Detect arch with uname,enops-1111
pr,rzpwqyytylqxowwlmywkpvpyqwlzuzyy,Create multi arch image,enops-1234
s,nsqzmntuqwqulqnxnwnxkypqtqklstov,Use alpha releaser to release releaser,
pr,tvqnnqqmvtmsqsvwootxswqvrowwxnrs,Empty commit to re-trigger CI,
```

### 3. Define Actions

Edit the file to specify what to do with each change:

| Action | Aliases | Description | Bookmark Handling |
|--------|---------|-------------|-------------------|
| `pr` | - | Create or update a PR for this change | 📝 Optional: Add in 4th column or leave empty for auto-generation |
| `skip` | `s` | Skip this change entirely | ❌ Not used |

**Bookmark behavior:**
- **Has bookmark + PR exists**: Keeps the existing PR in the stack
- **Has bookmark + no PR**: Creates a PR for that bookmark
- **User adds bookmark**: Creates the bookmark and PR
- **No bookmark**: jj automatically generates a bookmark name

### 4. Automatic Stacking

When you save and close the editor, `stack-prs` processes your changes **top-to-bottom**:

1. **First PR** → targets `main` (or your default branch)
2. **Second PR** → targets the first PR's branch
3. **Third PR** → targets the second PR's branch
4. And so on...

## Example Workflow

Given this edited file:

```csv
pr,abc123,Add user authentication,feature/auth
pr,def456,Add user profile page,feature/profile
pr,ghi789,Add profile settings,
s,jkl012,WIP: debugging,
```

`stack-prs` will:

1. ✅ Create bookmark `feature/auth` for change `abc123`
   - Push to GitHub
   - Create PR: `feature/auth` → `main`

2. ✅ Create bookmark `feature/profile` for change `def456`
   - Push to GitHub
   - Create PR: `feature/profile` → `feature/auth` (stacked!)

3. ✅ Auto-generate bookmark (e.g., `push-ghi789xyz`) for change `ghi789`
   - Push to GitHub
   - Create PR: `push-ghi789xyz` → `feature/profile` (stacked!)

4. ⏭️ Skip change `jkl012`

The result is a chain of dependent PRs:
```
main ← feature/auth ← feature/profile ← push-ghi789xyz
```

## Stack Navigation

After creating the PRs, `stack-prs` automatically adds a comment to each PR showing its position in the stack and linking to adjacent PRs:

**Example comment on the middle PR:**

```markdown
## Stack Information

This PR is **2 of 3** in the stack.

⬇️ Previous PR: [Add user authentication](https://github.com/owner/repo/pull/122)

⬆️ Next PR: [Add profile settings](https://github.com/owner/repo/pull/124)
```

**Key features:**
- 🔢 Shows position in the stack (e.g., "2 of 3")
- 🔗 Links to the previous PR (if not first)
- 🔗 Links to the next PR (if not last)
- 🔄 Updates automatically when you rerun the tool (no duplicate comments)

This makes it easy for reviewers to understand the context and navigate through related PRs.

## Tips

- **Review before running** - The interactive editor lets you review all changes before creating PRs
- **Add custom bookmark names** - Edit the 4th column to provide meaningful branch names, or leave empty for auto-generation
- **Use descriptive branch names** - They make the stack easy to navigate in GitHub
- **Reorder changes** - Edit the file to change the order of PRs in your stack
- **Keep existing PRs** - If a bookmark already has a PR, it will be kept in the stack automatically

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

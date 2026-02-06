# repo-check

A CLI tool to check if local Git repositories are safe to delete.

## Overview

`repo-check` scans Git repositories in a directory and determines whether each repository can be safely deleted without losing local-only work. It checks for:

- Uncommitted changes (working tree / index)
- Stash entries
- Local-only commits (commits not pushed to any remote)

## Installation

### From crates.io

```bash
cargo install repo-check
```

### From source

```bash
git clone https://github.com/petamorikei/repo-check.git
cd repo-check
cargo install --path .
```

## Usage

### Basic scan

```bash
# Scan repositories in current directory
repo-check

# Scan repositories in a specific directory
repo-check /path/to/workspace
```

### Output example

```
/home/user/projects/my-app [SAFE]
  - All checks passed

/home/user/projects/wip-feature [UNSAFE]
  - Uncommitted changes exist
  - Local-only commits exist
    Dirty files: 3
    Local-only commits: 2

/home/user/projects/local-only [UNKNOWN]
  - No remote tracking refs found

---
Summary: 3 total, 1 SAFE, 1 UNSAFE, 1 UNKNOWN
```

### Status definitions

| Status | Description |
|--------|-------------|
| **SAFE** | No local-only changes detected. Safe to delete. |
| **UNSAFE** | Local-only changes exist. Do not delete. |
| **UNKNOWN** | Cannot determine (e.g., no remote configured). Deletion not recommended. |

## Options

### Filtering

```bash
# Show only SAFE repositories
repo-check --only-safe

# Show only UNSAFE repositories
repo-check --only-unsafe

# Show only UNKNOWN repositories
repo-check --only-unknown
```

### Output format

```bash
# JSON output (for scripting)
repo-check --json
```

### Scan options

```bash
# Include current directory as a target
repo-check --include-dot

# Ignore untracked files when checking for uncommitted changes
repo-check --ignore-untracked
```

### Deletion

```bash
# Delete SAFE repositories (interactive confirmation)
repo-check --delete

# Delete without confirmation (for CI/scripts)
repo-check --delete --yes

# Move to trash instead of permanent deletion
repo-check --delete --trash

# Include UNKNOWN repositories in deletion candidates
repo-check --delete --allow-unknown
```

## Checks performed

### Check A: Uncommitted changes

Detects modified, staged, or untracked files using `git status --porcelain`.

### Check B: Stash entries

Detects stashed changes using `git stash list`.

### Check C: Local-only commits

Detects commits that exist in local branches but are not reachable from any remote tracking ref using `git log --branches --not --remotes`.

### Check D: Remote tracking refs

If no remote or remote tracking refs exist, the repository is marked as UNKNOWN since we cannot determine if commits are pushed.

## Limitations

- **No network operations**: Does not run `git fetch`. Remote tracking refs may be outdated.
- **Submodules not supported**: Only targets repositories where `.git` is a directory.
- **Worktrees not supported**: Linked worktrees are not detected.
- **LFS not checked**: Large File Storage push status is not verified.

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

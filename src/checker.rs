use crate::types::{Reason, RepoResult};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Execute a git command and return stdout
fn git_command(repo_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(["-C", repo_path.to_str().unwrap_or(".")])
        .args(args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Check A: Uncommitted changes (working tree / index)
pub fn check_uncommitted_changes(
    repo_path: &Path,
    result: &mut RepoResult,
    ignore_untracked: bool,
) {
    let output = match git_command(repo_path, &["status", "--porcelain"]) {
        Ok(o) => o,
        Err(e) => {
            result.mark_unknown(Reason::GitError(e.to_string()));
            result.errors.push(e.to_string());
            return;
        }
    };

    let mut dirty_count = 0;
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        // Untracked files start with '??'
        if ignore_untracked && line.starts_with("??") {
            continue;
        }
        dirty_count += 1;
    }

    result.dirty_count = dirty_count;
    if dirty_count > 0 {
        result.mark_unsafe(Reason::UncommittedChanges);
    }
}

/// Check B: Stash entries
pub fn check_stash(repo_path: &Path, result: &mut RepoResult) {
    let output = match git_command(repo_path, &["stash", "list"]) {
        Ok(o) => o,
        Err(e) => {
            result.mark_unknown(Reason::GitError(e.to_string()));
            result.errors.push(e.to_string());
            return;
        }
    };

    let stash_count = output.lines().filter(|l| !l.is_empty()).count();
    result.stash_count = stash_count;
    if stash_count > 0 {
        result.mark_unsafe(Reason::StashExists);
    }
}

/// Check C: Local-only commits (across all branches)
pub fn check_local_only_commits(repo_path: &Path, result: &mut RepoResult) {
    // First, check if remote tracking refs exist
    let remotes = match git_command(repo_path, &["remote"]) {
        Ok(o) => o,
        Err(e) => {
            result.mark_unknown(Reason::GitError(e.to_string()));
            result.errors.push(e.to_string());
            return;
        }
    };

    // Check if refs/remotes/* exists
    let remote_refs = match git_command(repo_path, &["for-each-ref", "--format=%(refname)", "refs/remotes/"]) {
        Ok(o) => o,
        Err(e) => {
            result.mark_unknown(Reason::GitError(e.to_string()));
            result.errors.push(e.to_string());
            return;
        }
    };

    if remotes.trim().is_empty() || remote_refs.trim().is_empty() {
        // No remote or no remote refs -> UNKNOWN
        result.mark_unknown(Reason::NoRemoteRefs);
        return;
    }

    // Detect commits that exist in local branches but not reachable from remotes
    // git log --oneline --branches --not --remotes
    let output = match git_command(repo_path, &["log", "--oneline", "--branches", "--not", "--remotes"]) {
        Ok(o) => o,
        Err(e) => {
            result.mark_unknown(Reason::GitError(e.to_string()));
            result.errors.push(e.to_string());
            return;
        }
    };

    let local_only_count = output.lines().filter(|l| !l.is_empty()).count();
    result.local_only_commit_count = local_only_count;
    if local_only_count > 0 {
        result.mark_unsafe(Reason::LocalOnlyCommits);
    }
}

/// Run all checks on a repository
pub fn check_repository(repo_path: &Path, ignore_untracked: bool) -> RepoResult {
    let mut result = RepoResult::new(repo_path.to_path_buf());

    // Check A: Uncommitted changes
    check_uncommitted_changes(repo_path, &mut result, ignore_untracked);

    // Check B: Stash
    check_stash(repo_path, &mut result);

    // Check C: Local-only commits (includes Check D)
    check_local_only_commits(repo_path, &mut result);

    // Add reason if SAFE
    result.finalize_safe();

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_git_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    #[test]
    fn test_clean_repo_no_remote() {
        let dir = setup_git_repo();
        fs::write(dir.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let result = check_repository(dir.path(), false);
        // No remote -> UNKNOWN
        assert_eq!(result.status, crate::types::Status::Unknown);
    }

    #[test]
    fn test_dirty_repo() {
        let dir = setup_git_repo();
        fs::write(dir.path().join("test.txt"), "hello").unwrap();

        let result = check_repository(dir.path(), false);
        assert_eq!(result.status, crate::types::Status::Unsafe);
        assert!(result.dirty_count > 0);
    }
}

use crate::checker::check_repository;
use crate::types::RepoResult;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// Check if a directory is a Git repository.
/// Only targets normal repositories where .git is a directory (excludes submodules).
fn is_git_repository(path: &Path) -> bool {
    let git_path = path.join(".git");
    git_path.exists() && git_path.is_dir()
}

/// Find Git repositories directly under the base path
pub fn find_repositories(base_path: &Path, include_dot: bool) -> Vec<std::path::PathBuf> {
    let mut repos = Vec::new();

    // Check the base directory itself (when --include-dot)
    if include_dot && is_git_repository(base_path) {
        repos.push(base_path.to_path_buf());
    }

    // Scan immediate subdirectories (depth=1)
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && is_git_repository(&path) {
                repos.push(path);
            }
        }
    }

    // Sort alphabetically
    repos.sort();
    repos
}

/// Scan all repositories (parallel execution, results in alphabetical order)
pub fn scan_repositories(
    base_path: &Path,
    include_dot: bool,
    ignore_untracked: bool,
) -> Vec<RepoResult> {
    let repos = find_repositories(base_path, include_dot);

    // Execute checks in parallel
    let mut results: Vec<RepoResult> = repos
        .par_iter()
        .map(|repo_path| check_repository(repo_path, ignore_untracked))
        .collect();

    // Sort alphabetically (parallel execution makes order non-deterministic)
    results.sort_by(|a, b| a.path.cmp(&b.path));

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_is_git_repository() {
        let dir = TempDir::new().unwrap();

        // Initially not a Git repository
        assert!(!is_git_repository(dir.path()));

        // After git init, it becomes a Git repository
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(is_git_repository(dir.path()));
    }

    #[test]
    fn test_find_repositories() {
        let base = TempDir::new().unwrap();

        // Create subdirectories and git init
        let repo1 = base.path().join("repo_a");
        let repo2 = base.path().join("repo_b");
        let not_repo = base.path().join("not_repo");

        fs::create_dir(&repo1).unwrap();
        fs::create_dir(&repo2).unwrap();
        fs::create_dir(&not_repo).unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(&repo1)
            .output()
            .unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(&repo2)
            .output()
            .unwrap();

        let repos = find_repositories(base.path(), false);
        assert_eq!(repos.len(), 2);
        assert!(repos[0].ends_with("repo_a"));
        assert!(repos[1].ends_with("repo_b"));
    }
}

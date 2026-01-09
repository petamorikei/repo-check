use crate::checker::check_repository;
use crate::types::RepoResult;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// ディレクトリがGitリポジトリかどうかを判定
/// .gitがディレクトリである通常のリポジトリのみ対象（サブモジュールは対象外）
fn is_git_repository(path: &Path) -> bool {
    let git_path = path.join(".git");
    git_path.exists() && git_path.is_dir()
}

/// カレントディレクトリ直下のGitリポジトリを検出
pub fn find_repositories(base_path: &Path, include_dot: bool) -> Vec<std::path::PathBuf> {
    let mut repos = Vec::new();

    // カレントディレクトリ自体をチェック（--include-dot時）
    if include_dot && is_git_repository(base_path) {
        repos.push(base_path.to_path_buf());
    }

    // 直下のディレクトリをスキャン（depth=1）
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && is_git_repository(&path) {
                repos.push(path);
            }
        }
    }

    // 辞書順でソート
    repos.sort();
    repos
}

/// 全リポジトリをスキャン（並列実行、結果は辞書順）
pub fn scan_repositories(
    base_path: &Path,
    include_dot: bool,
    ignore_untracked: bool,
) -> Vec<RepoResult> {
    let repos = find_repositories(base_path, include_dot);

    // 並列でチェック実行
    let mut results: Vec<RepoResult> = repos
        .par_iter()
        .map(|repo_path| check_repository(repo_path, ignore_untracked))
        .collect();

    // 辞書順でソート（並列実行で順序が不定になるため）
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

        // 初期状態ではGitリポジトリではない
        assert!(!is_git_repository(dir.path()));

        // git init後はGitリポジトリ
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

        // サブディレクトリを作成してgit init
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

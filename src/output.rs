use crate::types::{RepoResult, Status};
use colored::Colorize;

/// 単一リポジトリの結果を表示
fn print_repo_result(result: &RepoResult) {
    let path_str = result.path.display().to_string();
    let status_str = match result.status {
        Status::Safe => "SAFE".green().bold(),
        Status::Unsafe => "UNSAFE".red().bold(),
        Status::Unknown => "UNKNOWN".yellow().bold(),
    };

    println!("{} [{}]", path_str.bold(), status_str);

    // 理由を表示
    for reason in &result.reasons {
        println!("  - {}", reason);
    }

    // 補助情報
    if result.dirty_count > 0 {
        println!("    Dirty files: {}", result.dirty_count);
    }
    if result.stash_count > 0 {
        println!("    Stash entries: {}", result.stash_count);
    }
    if result.local_only_commit_count > 0 {
        println!("    Local-only commits: {}", result.local_only_commit_count);
    }

    // エラーがあれば表示
    for error in &result.errors {
        println!("    {}: {}", "Error".red(), error);
    }
}

/// サマリーを表示
fn print_summary(results: &[RepoResult]) {
    let safe_count = results.iter().filter(|r| r.status == Status::Safe).count();
    let unsafe_count = results
        .iter()
        .filter(|r| r.status == Status::Unsafe)
        .count();
    let unknown_count = results
        .iter()
        .filter(|r| r.status == Status::Unknown)
        .count();

    println!("---");
    println!(
        "Summary: {} total, {} {}, {} {}, {} {}",
        results.len(),
        safe_count,
        "SAFE".green(),
        unsafe_count,
        "UNSAFE".red(),
        unknown_count,
        "UNKNOWN".yellow()
    );
}

/// フィルタリングして出力
pub fn print_filtered(results: &[RepoResult], filter: Option<Status>, json: bool) {
    let filtered: Vec<&RepoResult> = match filter {
        Some(status) => results.iter().filter(|r| r.status == status).collect(),
        None => results.iter().collect(),
    };

    if json {
        let json_str =
            serde_json::to_string_pretty(&filtered).unwrap_or_else(|_| "[]".to_string());
        println!("{}", json_str);
    } else {
        if filtered.is_empty() {
            println!("No repositories match the filter.");
            return;
        }
        for result in &filtered {
            print_repo_result(result);
            println!();
        }

        // フィルタリング時もサマリー表示（全体の統計）
        print_summary(results);
    }
}

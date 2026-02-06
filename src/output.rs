use crate::types::{RepoResult, Status};
use colored::Colorize;

/// Display result for a single repository
fn print_repo_result(result: &RepoResult) {
    let path_str = result.path.display().to_string();
    let status_str = match result.status {
        Status::Safe => "SAFE".green().bold(),
        Status::Unsafe => "UNSAFE".red().bold(),
        Status::Unknown => "UNKNOWN".yellow().bold(),
    };

    println!("{} [{}]", path_str.bold(), status_str);

    // Display reasons
    for reason in &result.reasons {
        println!("  - {}", reason);
    }

    // Auxiliary information
    if result.dirty_count > 0 {
        println!("    Dirty files: {}", result.dirty_count);
    }
    if result.stash_count > 0 {
        println!("    Stash entries: {}", result.stash_count);
    }
    if result.local_only_commit_count > 0 {
        println!("    Local-only commits: {}", result.local_only_commit_count);
    }

    // Display errors if any
    for error in &result.errors {
        println!("    {}: {}", "Error".red(), error);
    }
}

/// Display summary
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

/// Filter and output results
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

        // Show summary even when filtering (overall statistics)
        print_summary(results);
    }
}

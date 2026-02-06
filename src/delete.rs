use crate::checker;
use crate::types::{DeleteConfirm, RepoResult, Status};
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::Path;

/// Filter repositories that are candidates for deletion
pub fn get_delete_candidates(
    results: &[RepoResult],
    allow_unknown: bool,
) -> Vec<&RepoResult> {
    results
        .iter()
        .filter(|r| {
            r.status == Status::Safe || (allow_unknown && r.status == Status::Unknown)
        })
        .collect()
}

/// Delete a repository (prefer trash, fallback to rm -rf)
fn delete_repository(path: &Path, use_trash: bool, skip_confirm: bool) -> Result<bool> {
    if use_trash {
        match trash::delete(path) {
            Ok(()) => return Ok(true),
            Err(e) => {
                if skip_confirm {
                    eprintln!(
                        "{}: Failed to move to trash ({}), skipping (use without --trash to force rm -rf)",
                        "Warning".yellow(),
                        e
                    );
                    return Ok(false);
                }
                eprintln!(
                    "{}: Failed to move to trash: {}",
                    "Warning".yellow(),
                    e
                );
                let options = &["Yes, use rm -rf instead", "No, skip this repository"];
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Fall back to permanent deletion?")
                    .items(options)
                    .default(1)
                    .interact_opt();
                match selection {
                    Ok(Some(0)) => {} // fall through to rm -rf
                    _ => return Ok(false),
                }
            }
        }
    }

    fs::remove_dir_all(path)?;
    Ok(true)
}

/// Ask user for deletion confirmation
fn ask_confirmation(path: &Path) -> DeleteConfirm {
    let path_str = path.display().to_string();
    println!("\nDelete {}?", path_str.bold());

    let options = &["Yes", "No", "All (delete all remaining)", "Quit"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(options)
        .default(1) // Default is "No"
        .interact_opt();

    match selection {
        Ok(Some(0)) => DeleteConfirm::Yes,
        Ok(Some(1)) => DeleteConfirm::No,
        Ok(Some(2)) => DeleteConfirm::All,
        Ok(Some(3)) => DeleteConfirm::Quit,
        _ => DeleteConfirm::No,
    }
}

/// Execute deletion
pub fn execute_delete(
    candidates: &[&RepoResult],
    use_trash: bool,
    skip_confirm: bool,
) -> Result<(usize, usize)> {
    let mut deleted = 0;
    let mut skipped = 0;
    let mut delete_all = skip_confirm;

    for result in candidates {
        let path = &result.path;

        if !delete_all {
            match ask_confirmation(path) {
                DeleteConfirm::Yes => {}
                DeleteConfirm::No => {
                    skipped += 1;
                    continue;
                }
                DeleteConfirm::All => {
                    delete_all = true;
                }
                DeleteConfirm::Quit => {
                    println!("Aborted.");
                    break;
                }
            }
        }

        // TOCTOU mitigation: recheck before deletion
        if !checker::quick_recheck(path) {
            println!(
                "{}: Repository state changed since scan, skipping: {}",
                "Warning".yellow(),
                path.display()
            );
            skipped += 1;
            continue;
        }

        // Execute deletion
        print!("Deleting {}... ", path.display());
        match delete_repository(path, use_trash, delete_all) {
            Ok(true) => {
                println!("{}", "done".green());
                deleted += 1;
            }
            Ok(false) => {
                skipped += 1;
            }
            Err(e) => {
                println!("{}: {}", "failed".red(), e);
                skipped += 1;
            }
        }
    }

    Ok((deleted, skipped))
}

/// Display deletion candidates
pub fn show_delete_candidates(candidates: &[&RepoResult]) {
    if candidates.is_empty() {
        println!("No repositories to delete.");
        return;
    }

    println!("The following repositories will be deleted:\n");
    for result in candidates {
        let status_str = match result.status {
            Status::Safe => "SAFE".green(),
            Status::Unknown => "UNKNOWN".yellow(),
            _ => "?".normal(),
        };
        println!("  {} [{}]", result.path.display(), status_str);
    }
    println!("\nTotal: {} repositories", candidates.len());
}

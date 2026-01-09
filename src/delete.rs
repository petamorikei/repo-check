use crate::types::{DeleteConfirm, RepoResult, Status};
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::Path;

/// 削除対象のリポジトリをフィルタリング
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

/// リポジトリを削除（ゴミ箱優先）
fn delete_repository(path: &Path, use_trash: bool) -> Result<()> {
    if use_trash {
        // ゴミ箱への移動を試みる
        match trash::delete(path) {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!(
                    "{}: Failed to move to trash ({}), falling back to rm -rf",
                    "Warning".yellow(),
                    e
                );
            }
        }
    }

    // rm -rf にフォールバック
    fs::remove_dir_all(path)?;
    Ok(())
}

/// ユーザーに削除確認を求める
fn ask_confirmation(path: &Path) -> DeleteConfirm {
    let path_str = path.display().to_string();
    println!("\nDelete {}?", path_str.bold());

    let options = &["Yes", "No", "All (delete all remaining)", "Quit"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(options)
        .default(1) // デフォルトは "No"
        .interact_opt();

    match selection {
        Ok(Some(0)) => DeleteConfirm::Yes,
        Ok(Some(1)) => DeleteConfirm::No,
        Ok(Some(2)) => DeleteConfirm::All,
        Ok(Some(3)) => DeleteConfirm::Quit,
        _ => DeleteConfirm::No,
    }
}

/// 削除を実行
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

        // 削除実行
        print!("Deleting {}... ", path.display());
        match delete_repository(path, use_trash) {
            Ok(()) => {
                println!("{}", "done".green());
                deleted += 1;
            }
            Err(e) => {
                println!("{}: {}", "failed".red(), e);
                skipped += 1;
            }
        }
    }

    Ok((deleted, skipped))
}

/// 削除候補を表示
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

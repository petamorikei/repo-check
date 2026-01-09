mod checker;
mod cli;
mod delete;
mod output;
mod scanner;
mod types;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use std::path::Path;
use types::Status;

fn main() -> Result<()> {
    let args = Args::parse();

    let base_path = Path::new(&args.path).canonicalize()?;

    // リポジトリをスキャン
    let results = scanner::scan_repositories(&base_path, args.include_dot, args.ignore_untracked);

    // フィルタを決定
    let filter = if args.only_safe {
        Some(Status::Safe)
    } else if args.only_unsafe {
        Some(Status::Unsafe)
    } else if args.only_unknown {
        Some(Status::Unknown)
    } else {
        None
    };

    // 削除モード
    if args.delete {
        let candidates = delete::get_delete_candidates(&results, args.allow_unknown);

        if candidates.is_empty() {
            println!("No repositories to delete.");
            return Ok(());
        }

        // 削除候補を表示
        delete::show_delete_candidates(&candidates);

        // 削除実行
        let (deleted, skipped) = delete::execute_delete(&candidates, args.trash, args.yes)?;

        println!("\nDeleted: {}, Skipped: {}", deleted, skipped);
    } else {
        // スキャンのみモード
        output::print_filtered(&results, filter, args.json);
    }

    Ok(())
}

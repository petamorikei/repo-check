mod checker;
mod cli;
mod delete;
mod output;
mod scanner;
mod types;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Args;
use std::path::Path;
use types::Status;

fn main() -> Result<()> {
    let args = Args::parse();

    let base_path = Path::new(&args.path)
        .canonicalize()
        .context(format!("Failed to resolve path: {}", args.path))?;

    // Scan repositories
    let results = scanner::scan_repositories(&base_path, args.include_dot, args.ignore_untracked);

    // Determine filter
    let filter = if args.only_safe {
        Some(Status::Safe)
    } else if args.only_unsafe {
        Some(Status::Unsafe)
    } else if args.only_unknown {
        Some(Status::Unknown)
    } else {
        None
    };

    // Delete mode
    if args.delete {
        let candidates = delete::get_delete_candidates(&results, args.allow_unknown);

        if candidates.is_empty() {
            println!("No repositories to delete.");
            return Ok(());
        }

        delete::show_delete_candidates(&candidates);

        if args.dry_run {
            println!("\n(dry-run mode: no repositories were deleted)");
        } else {
            let (deleted, skipped) =
                delete::execute_delete(&candidates, args.trash, args.yes)?;
            println!("\nDeleted: {}, Skipped: {}", deleted, skipped);
        }
    } else {
        // Scan-only mode
        output::print_filtered(&results, filter, args.json);
    }

    Ok(())
}

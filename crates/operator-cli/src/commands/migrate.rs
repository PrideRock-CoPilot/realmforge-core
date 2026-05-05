use clap::{Args, Subcommand};
use serde_json::json;
use std::path::PathBuf;

use crate::util::{output_json, CliResult};

/// Database migration commands.
#[derive(Subcommand)]
pub enum MigrateCommand {
    Up,
    Down,
    List(ListArgs),
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, default_value = "db/migrations")]
    pub path: PathBuf,
}

pub async fn handle_migrate(cmd: MigrateCommand) -> CliResult {
    match cmd {
        MigrateCommand::Up => {
            output_json(&json!({"status": "migrate up — requires database connection"}));
        }
        MigrateCommand::Down => {
            output_json(&json!({"status": "migrate down — requires database connection"}));
        }
        MigrateCommand::List(args) => {
            let entries = std::fs::read_dir(&args.path)
                .map_err(|e| format!("Cannot read migration path {}: {}", args.path.display(), e))?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "sql"))
                .collect::<Vec<_>>();
            output_json(
                &json!({"migrations": entries.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>()}),
            );
        }
    }
    Ok(())
}

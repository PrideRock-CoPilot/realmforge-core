use clap::{Parser, Subcommand};
use rf_snapshot::SnapshotManifest;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "realmforge")]
#[command(about = "RealmForge Core operator CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Version,
    ListMigrations {
        #[arg(long, default_value = "db/migrations")]
        path: PathBuf,
    },
    ValidateManifest {
        path: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Version => {
            println!("realmforge {}", env!("CARGO_PKG_VERSION"));
        }
        Command::ListMigrations { path } => {
            let mut entries = fs::read_dir(path)?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
                .collect::<Vec<_>>();
            entries.sort();
            for entry in entries {
                println!("{}", entry.display());
            }
        }
        Command::ValidateManifest { path } => {
            let content = fs::read_to_string(path)?;
            let manifest: SnapshotManifest = serde_json::from_str(&content)?;
            if manifest.verify_hash()? {
                println!("valid {}", manifest.manifest_hash);
            } else {
                println!("invalid {}", manifest.manifest_hash);
                std::process::exit(2);
            }
        }
    }
    Ok(())
}

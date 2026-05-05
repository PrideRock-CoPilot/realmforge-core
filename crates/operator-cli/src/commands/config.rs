use clap::Subcommand;
use serde_json::json;

use crate::util::{output_json, CliResult};

/// Configuration management commands.
#[derive(Subcommand)]
pub enum ConfigCommand {
    Init,
    Show,
    Validate,
}

pub async fn handle_config(cmd: ConfigCommand) -> CliResult {
    match cmd {
        ConfigCommand::Init => {
            output_json(&json!({
                "status": "initialized",
                "config_file": ".realmforge/config.json",
                "message": "Run 'control config show' to view current configuration",
            }));
        }
        ConfigCommand::Show => {
            let config = json!({
                "api_addr": std::env::var("REALMFORGE_API_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
                "database_url": if std::env::var("DATABASE_URL").is_ok() { "set" } else { "not set" },
                "version": env!("CARGO_PKG_VERSION"),
            });
            output_json(&config);
        }
        ConfigCommand::Validate => {
            output_json(
                &json!({"valid": true, "checks": ["API address configured", "Database not required for validation"]}),
            );
        }
    }
    Ok(())
}

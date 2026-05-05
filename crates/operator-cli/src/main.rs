mod commands;
mod util;

use clap::{Parser, Subcommand};
use commands::{
    actor::ActorCommand, audit::AuditCommand, boards::BoardsCommand, bundle::BundleCommand,
    catalog::CatalogCommand, command::CommandCommand, config::ConfigCommand,
    grant::GrantCommand, knowledge::KnowledgeCommand, live_watch::LiveWatchCommands,
    login::LoginCommand, migrate::MigrateCommand, rollback::RollbackCommand,
    runtime::RuntimeCommand, session::SessionCommand, skill::SkillCommand,
    snapshot::SnapshotCommand, watch::WatchCommand, work_packet::WorkPacketCommand,
    work_path::WorkPathCommand,
};

use control_service::ServiceContext;
use util::CliResult;

#[derive(Parser)]
#[command(name = "control", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Operator control CLI")]
struct Cli {
    #[arg(global = true, short, long)]
    pretty: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Command {
    /// Show version information
    Version,
    /// Session lifecycle management
    Session {
        #[command(subcommand)]
        cmd: SessionCommand,
    },
    /// Command lifecycle management
    Command {
        #[command(subcommand)]
        cmd: CommandCommand,
    },
    /// Audit trail query and verification
    Audit {
        #[command(subcommand)]
        cmd: AuditCommand,
    },
    /// Snapshot management
    Snapshot {
        #[command(subcommand)]
        cmd: SnapshotCommand,
    },
    /// Rollback management
    Rollback {
        #[command(subcommand)]
        cmd: RollbackCommand,
    },
    /// Actor scope management
    Actor {
        #[command(subcommand)]
        cmd: ActorCommand,
    },
    /// Work packet generation and validation
    WorkPacket {
        #[command(subcommand)]
        cmd: WorkPacketCommand,
    },
    /// Skill registration and management
    Skill {
        #[command(subcommand)]
        cmd: SkillCommand,
    },
    /// Database migration management
    Migrate {
        #[command(subcommand)]
        cmd: MigrateCommand,
    },
    /// Catalog module management
    Catalog {
        #[command(subcommand)]
        cmd: CatalogCommand,
    },
    /// Work path graph management
    WorkPath {
        #[command(subcommand)]
        cmd: WorkPathCommand,
    },
    /// Skill grant lifecycle management
    Grant {
        #[command(subcommand)]
        cmd: GrantCommand,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        cmd: ConfigCommand,
    },
    /// Knowledge lifecycle management
    Knowledge {
        #[command(subcommand)]
        cmd: KnowledgeCommand,
    },
    /// Board planning and approvals management
    Boards {
        #[command(subcommand)]
        cmd: BoardsCommand,
    },
    /// Build watch event monitoring and cost tracking
    Watch {
        #[command(subcommand)]
        cmd: WatchCommand,
    },
    /// Live Watch runtime health monitoring and remediation
    LiveWatch {
        #[command(subcommand)]
        cmd: LiveWatchCommands,
    },
    /// Bundle lifecycle management
    Bundle {
        #[command(subcommand)]
        cmd: BundleCommand,
    },
    /// Runtime instance management
    Runtime {
        #[command(subcommand)]
        cmd: RuntimeCommand,
    },
    /// Login authentication and policy management
    Login {
        #[command(subcommand)]
        cmd: LoginCommand,
    },
}

fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let cli = Cli::parse();

    // For commands that don't need a store, handle them early
    match &cli.command {
        Command::Version => {
            println!("control {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Command::Config { .. } | Command::Migrate { .. } => {
            // These commands handle their own store-less execution
        }
        _ => {}
    }

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/realmforge".to_string());
    let ctx = ServiceContext::connect(&db_url).await.expect(
        "DATABASE_URL not reachable — set DATABASE_URL to a valid PostgreSQL connection string",
    );

    let result: CliResult = match cli.command {
        Command::Version => Ok(()),
        Command::Session { cmd } => session::handle_session(cmd, &ctx, cli.pretty).await,
        Command::Command { cmd } => command::handle_command(cmd, &ctx).await,
        Command::Audit { cmd } => audit::handle_audit(cmd, &ctx).await,
        Command::Snapshot { cmd } => snapshot::handle_snapshot(cmd, &ctx).await,
        Command::Rollback { cmd } => rollback::handle_rollback(cmd, &ctx).await,
        Command::Actor { cmd } => actor::handle_actor(cmd, &ctx).await,
        Command::WorkPacket { cmd } => work_packet::handle_work_packet(cmd, &ctx).await,
        Command::Skill { cmd } => skill::handle_skill(cmd, &ctx).await,
        Command::Catalog { cmd } => catalog::handle_catalog(cmd, &ctx).await,
        Command::WorkPath { cmd } => work_path::handle_work_path(cmd, &ctx).await,
        Command::Migrate { cmd } => migrate::handle_migrate(cmd).await,
        Command::Grant { cmd } => grant::handle_grant(cmd, &ctx).await,
        Command::Config { cmd } => config::handle_config(cmd).await,
        Command::Knowledge { cmd } => knowledge::handle_knowledge(cmd, &ctx).await,
        Command::Boards { cmd } => boards::handle_boards(cmd, &ctx).await,
        Command::Watch { cmd } => watch::handle_watch(cmd, &ctx).await,
        Command::LiveWatch { cmd } => live_watch::handle_live_watch(cmd, &ctx).await,
        Command::Bundle { cmd } => bundle::handle_bundle(cmd, &ctx).await,
        Command::Login { cmd } => login::handle_login(cmd, &ctx, cli.pretty).await,
        Command::Runtime { cmd } => runtime::handle_runtime(cmd, &ctx).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

mod session {
    use super::*;
    pub async fn handle_session(
        cmd: SessionCommand,
        ctx: &ServiceContext,
        pretty: bool,
    ) -> CliResult {
        commands::session::handle_session(cmd, ctx, pretty).await
    }
}

mod command {
    use super::*;
    pub async fn handle_command(cmd: CommandCommand, ctx: &ServiceContext) -> CliResult {
        commands::command::handle_command(cmd, ctx).await
    }
}

mod audit {
    use super::*;
    pub async fn handle_audit(cmd: AuditCommand, ctx: &ServiceContext) -> CliResult {
        commands::audit::handle_audit(cmd, ctx).await
    }
}

mod snapshot {
    use super::*;
    pub async fn handle_snapshot(cmd: SnapshotCommand, ctx: &ServiceContext) -> CliResult {
        commands::snapshot::handle_snapshot(cmd, ctx).await
    }
}

mod rollback {
    use super::*;
    pub async fn handle_rollback(cmd: RollbackCommand, ctx: &ServiceContext) -> CliResult {
        commands::rollback::handle_rollback(cmd, ctx).await
    }
}

mod actor {
    use super::*;
    pub async fn handle_actor(cmd: ActorCommand, ctx: &ServiceContext) -> CliResult {
        commands::actor::handle_actor(cmd, ctx).await
    }
}

mod work_packet {
    use super::*;
    pub async fn handle_work_packet(cmd: WorkPacketCommand, ctx: &ServiceContext) -> CliResult {
        commands::work_packet::handle_work_packet(cmd, ctx).await
    }
}

mod skill {
    use super::*;
    pub async fn handle_skill(cmd: SkillCommand, ctx: &ServiceContext) -> CliResult {
        commands::skill::handle_skill(cmd, ctx).await
    }
}

mod migrate {
    use super::*;
    pub async fn handle_migrate(cmd: MigrateCommand) -> CliResult {
        commands::migrate::handle_migrate(cmd).await
    }
}

mod config {
    use super::*;
    pub async fn handle_config(cmd: ConfigCommand) -> CliResult {
        commands::config::handle_config(cmd).await
    }
}

mod catalog {
    use super::*;
    pub async fn handle_catalog(cmd: CatalogCommand, ctx: &ServiceContext) -> CliResult {
        commands::catalog::handle_catalog(cmd, ctx).await
    }
}

mod work_path {
    use super::*;
    pub async fn handle_work_path(cmd: WorkPathCommand, ctx: &ServiceContext) -> CliResult {
        commands::work_path::handle_work_path(cmd, ctx).await
    }
}

mod grant {
    use super::*;
    pub async fn handle_grant(cmd: GrantCommand, ctx: &ServiceContext) -> CliResult {
        commands::grant::handle_grant(cmd, ctx).await
    }
}

mod knowledge {
    use super::*;
    pub async fn handle_knowledge(cmd: KnowledgeCommand, ctx: &ServiceContext) -> CliResult {
        commands::knowledge::handle_knowledge(cmd, ctx).await
    }
}

mod boards {
    use super::*;
    pub async fn handle_boards(cmd: BoardsCommand, ctx: &ServiceContext) -> CliResult {
        commands::boards::handle_boards(cmd, ctx).await
    }
}

mod watch {
    use super::*;
    pub async fn handle_watch(cmd: WatchCommand, ctx: &ServiceContext) -> CliResult {
        commands::watch::handle_watch(cmd, ctx).await
    }
}

mod live_watch {
    use super::*;
    pub async fn handle_live_watch(cmd: LiveWatchCommands, ctx: &ServiceContext) -> CliResult {
        commands::live_watch::execute(cmd, ctx).await
    }
}

mod bundle {
    use super::*;
    pub async fn handle_bundle(cmd: BundleCommand, ctx: &ServiceContext) -> CliResult {
        commands::bundle::handle_bundle(cmd, ctx).await
    }
}

mod runtime {
    use super::*;
    pub async fn handle_runtime(cmd: RuntimeCommand, ctx: &ServiceContext) -> CliResult {
        commands::runtime::handle_runtime(cmd, ctx).await
    }
}

mod login {
    use super::*;
    pub async fn handle_login(cmd: LoginCommand, ctx: &ServiceContext, pretty: bool) -> CliResult {
        commands::login::handle_login_command(cmd, ctx, pretty).await
    }
}

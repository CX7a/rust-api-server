mod commands;
mod config;
mod client;
mod error;
mod utils;

use clap::{Parser, Subcommand};
use tracing::Level;

#[derive(Parser)]
#[command(
    name = "cx7",
    version = "0.1.0",
    about = "CompileX7 CLI - Local development toolkit",
    long_about = "A comprehensive command-line interface for managing CompileX7 projects, deployments, and agents"
)]
#[command(propagate_version = true)]
struct Cli {
    /// Backend server URL
    #[arg(global = true, long, env = "CX7_SERVER")]
    server: Option<String>,

    /// Enable debug output
    #[arg(global = true, long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication commands (login, logout, status)
    Auth(commands::auth::AuthArgs),

    /// Project management commands
    Project(commands::project::ProjectArgs),

    /// Deploy code and manage deployments
    Deploy(commands::deploy::DeployArgs),

    /// Configuration management
    Config(commands::config::ConfigArgs),

    /// Agent management and execution
    Agent(commands::agent::AgentArgs),

    /// System and service status
    Status(commands::status::StatusArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let level = if cli.debug { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .compact()
        .init();

    // Initialize config
    let mut cfg = config::Config::load().await?;
    if let Some(server) = cli.server {
        cfg.server_url = server;
    }

    // Execute command
    match cli.command {
        Commands::Auth(args) => commands::auth::execute(cfg, args).await?,
        Commands::Project(args) => commands::project::execute(cfg, args).await?,
        Commands::Deploy(args) => commands::deploy::execute(cfg, args).await?,
        Commands::Config(args) => commands::config::execute(cfg, args).await?,
        Commands::Agent(args) => commands::agent::execute(cfg, args).await?,
        Commands::Status(args) => commands::status::execute(cfg, args).await?,
    }

    Ok(())
}

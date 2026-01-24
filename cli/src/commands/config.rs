use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::utils;
use colored::*;

#[derive(Parser)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Show configuration
    Show,
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Reset to defaults
    Reset {
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

pub async fn execute(mut config: Config, args: ConfigArgs) -> anyhow::Result<()> {
    match args.command {
        ConfigCommand::Show => show_config(&config),
        ConfigCommand::Set { key, value } => set_config(&mut config, &key, &value).await,
        ConfigCommand::Get { key } => get_config(&config, &key),
        ConfigCommand::Reset { force } => reset_config(&mut config, force).await,
    }
}

fn show_config(config: &Config) -> anyhow::Result<()> {
    println!("{}", "Configuration:".bold());
    println!("  Server: {}", config.server_url.cyan());
    println!("  Email: {}", config.user_email.as_deref().unwrap_or("Not set").cyan());
    println!("  Auth Token: {}***", &config.auth_token[..config.auth_token.len().min(8)].cyan());
    Ok(())
}

fn get_config(config: &Config, key: &str) -> anyhow::Result<()> {
    let value = match key {
        "server" => config.server_url.clone(),
        "email" => config.user_email.clone().unwrap_or_else(|| "Not set".to_string()),
        "token" => format!("{}***", &config.auth_token[..config.auth_token.len().min(8)]),
        _ => return Err(anyhow::anyhow!("Unknown key: {}", key)),
    };
    println!("{}: {}", key, value.cyan());
    Ok(())
}

async fn set_config(config: &mut Config, key: &str, value: &str) -> anyhow::Result<()> {
    match key {
        "server" => config.server_url = value.to_string(),
        "email" => config.user_email = Some(value.to_string()),
        _ => return Err(anyhow::anyhow!("Unknown key: {}", key)),
    }
    config.save().await?;
    println!("{}", format!("✓ {} set to {}", key, value).green().bold());
    Ok(())
}

async fn reset_config(config: &mut Config, force: bool) -> anyhow::Result<()> {
    if !force {
        let confirm = utils::confirm("Reset configuration to defaults? This will clear all settings.");
        if !confirm {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    *config = Config::default();
    config.save().await?;
    println!("{}", "✓ Configuration reset to defaults".green().bold());
    Ok(())
}

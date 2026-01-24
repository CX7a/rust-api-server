use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::utils;
use colored::*;

#[derive(Parser)]
pub struct AuthArgs {
    #[command(subcommand)]
    command: AuthCommand,
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Login to CompileX7
    Login {
        /// Email address
        #[arg(short, long)]
        email: Option<String>,
    },
    /// Logout from CompileX7
    Logout,
    /// Show current authenticated user
    Whoami,
    /// Refresh authentication token
    Refresh,
}

pub async fn execute(mut config: Config, args: AuthArgs) -> anyhow::Result<()> {
    match args.command {
        AuthCommand::Login { email } => login(config, email).await,
        AuthCommand::Logout => logout(config).await,
        AuthCommand::Whoami => whoami(config).await,
        AuthCommand::Refresh => refresh_token(config).await,
    }
}

async fn login(mut config: Config, email: Option<String>) -> anyhow::Result<()> {
    let email = email.unwrap_or_else(|| {
        utils::prompt("Email: ")
    });

    let password = rpassword::prompt_password("Password: ")?;

    utils::spinner_start("Authenticating...");

    let client = crate::client::ApiClient::new(&config.server_url, None);
    match client.login(&email, &password).await {
        Ok(response) => {
            config.auth_token = response.token;
            config.user_email = Some(email.clone());
            config.save().await?;
            
            utils::spinner_stop();
            println!("{}", format!("✓ Successfully logged in as {}", email).green().bold());
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Login failed: {}", e))
        }
    }
}

async fn logout(config: Config) -> anyhow::Result<()> {
    let mut cfg = config;
    cfg.auth_token = String::new();
    cfg.user_email = None;
    cfg.save().await?;
    println!("{}", "✓ Successfully logged out".green().bold());
    Ok(())
}

async fn whoami(config: Config) -> anyhow::Result<()> {
    if config.auth_token.is_empty() {
        println!("{}", "Not authenticated. Run 'cx7 auth login' to authenticate.".yellow());
        return Ok(());
    }

    utils::spinner_start("Fetching user info...");
    
    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.get_user_info().await {
        Ok(user) => {
            utils::spinner_stop();
            println!("{}", "Current User:".bold());
            println!("  Email: {}", user.email);
            println!("  ID: {}", user.id);
            println!("  Created: {}", user.created_at);
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to fetch user info: {}", e))
        }
    }
}

async fn refresh_token(mut config: Config) -> anyhow::Result<()> {
    if config.auth_token.is_empty() {
        println!("{}", "Not authenticated. Run 'cx7 auth login' first.".yellow());
        return Ok(());
    }

    utils::spinner_start("Refreshing token...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.refresh_token().await {
        Ok(response) => {
            config.auth_token = response.token;
            config.save().await?;
            
            utils::spinner_stop();
            println!("{}", "✓ Token refreshed successfully".green().bold());
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Token refresh failed: {}", e))
        }
    }
}

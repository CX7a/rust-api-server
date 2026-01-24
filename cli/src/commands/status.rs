use clap::Parser;
use crate::config::Config;
use crate::utils;
use colored::*;

#[derive(Parser)]
pub struct StatusArgs {
    /// Show detailed status
    #[arg(short, long)]
    detail: bool,
}

pub async fn execute(config: Config, args: StatusArgs) -> anyhow::Result<()> {
    utils::spinner_start("Checking status...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    
    match client.health_check().await {
        Ok(health) => {
            utils::spinner_stop();
            
            let server_status = if health.ok { "Online".green() } else { "Offline".red() };
            println!("{}", format!("Server: {}", server_status).bold());

            if args.detail {
                println!("\nDetailed Status:");
                println!("  Database: {}", health.database_ok);
                println!("  Cache: {}", health.cache_ok);
                println!("  Agents: {}", health.agents_running);
            }

            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            println!("{}", format!("Server: Offline").red().bold());
            Err(anyhow::anyhow!("Health check failed: {}", e))
        }
    }
}

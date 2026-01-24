use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::utils;
use colored::*;

#[derive(Parser)]
pub struct AgentArgs {
    #[command(subcommand)]
    command: AgentCommand,
}

#[derive(Subcommand)]
enum AgentCommand {
    /// List available agents
    List,
    /// Run an agent
    Run {
        /// Agent name (backend/frontend/qa)
        agent: String,
        /// Project ID or name
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Check agent status
    Status {
        /// Agent name
        agent: String,
    },
}

pub async fn execute(config: Config, args: AgentArgs) -> anyhow::Result<()> {
    if config.auth_token.is_empty() {
        println!("{}", "Not authenticated. Run 'cx7 auth login' first.".red());
        return Ok(());
    }

    match args.command {
        AgentCommand::List => list_agents(config).await,
        AgentCommand::Run { agent, project } => run_agent(config, &agent, project).await,
        AgentCommand::Status { agent } => check_status(config, &agent).await,
    }
}

async fn list_agents(config: Config) -> anyhow::Result<()> {
    utils::spinner_start("Fetching agents...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.list_agents().await {
        Ok(agents) => {
            utils::spinner_stop();
            println!("{}", "Available Agents:".bold());
            for agent in agents {
                println!("  {} - {}", agent.name.cyan(), agent.description);
            }
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to fetch agents: {}", e))
        }
    }
}

async fn run_agent(config: Config, agent: &str, project: Option<String>) -> anyhow::Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());

    utils::spinner_start(&format!("Running {} agent...", agent));

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.run_agent(&project, agent).await {
        Ok(result) => {
            utils::spinner_stop();
            println!("{}", format!("✓ {} agent completed", agent).green().bold());
            println!("  Result ID: {}", result.id);
            println!("  Status: {}", result.status);
            if let Some(output) = result.output {
                println!("  Output:\n{}", output);
            }
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Agent execution failed: {}", e))
        }
    }
}

async fn check_status(config: Config, agent: &str) -> anyhow::Result<()> {
    utils::spinner_start("Checking status...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.get_agent_status(agent).await {
        Ok(status) => {
            utils::spinner_stop();
            println!("{}", format!("{}: {}", agent, status.status).cyan());
            println!("  Last Run: {}", status.last_run);
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to check status: {}", e))
        }
    }
}

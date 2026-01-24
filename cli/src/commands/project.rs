use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::utils;
use colored::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Parser)]
pub struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommand,
}

#[derive(Subcommand)]
enum ProjectCommand {
    /// Initialize a new project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List all projects
    List {
        /// Show detailed information
        #[arg(short, long)]
        detail: bool,
    },
    /// Show project details
    Show {
        /// Project ID or name
        project: String,
    },
    /// Create a new project
    Create {
        /// Project name
        #[arg(short, long)]
        name: String,
        /// Project description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete a project
    Delete {
        /// Project ID or name
        project: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

pub async fn execute(config: Config, args: ProjectArgs) -> anyhow::Result<()> {
    if config.auth_token.is_empty() {
        println!("{}", "Not authenticated. Run 'cx7 auth login' first.".red());
        return Ok(());
    }

    match args.command {
        ProjectCommand::Init { name } => init_project(config, name).await,
        ProjectCommand::List { detail } => list_projects(config, detail).await,
        ProjectCommand::Show { project } => show_project(config, project).await,
        ProjectCommand::Create { name, description } => create_project(config, name, description).await,
        ProjectCommand::Delete { project, force } => delete_project(config, project, force).await,
    }
}

async fn init_project(config: Config, name: Option<String>) -> anyhow::Result<()> {
    let name = name.unwrap_or_else(|| {
        utils::prompt("Project name: ")
    });

    utils::spinner_start("Initializing project...");

    let project_dir = std::env::current_dir()?;
    let cx7_dir = project_dir.join(".cx7");
    std::fs::create_dir_all(&cx7_dir)?;

    let project_config = ProjectConfig {
        name: name.clone(),
        id: Uuid::new_v4().to_string(),
        created_at: chrono::Utc::now(),
    };

    let config_path = cx7_dir.join("project.toml");
    let config_str = toml::to_string_pretty(&project_config)?;
    std::fs::write(config_path, config_str)?;

    utils::spinner_stop();
    println!("{}", format!("✓ Project '{}' initialized", name).green().bold());
    Ok(())
}

async fn list_projects(config: Config, detail: bool) -> anyhow::Result<()> {
    utils::spinner_start("Fetching projects...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.list_projects().await {
        Ok(projects) => {
            utils::spinner_stop();
            
            if projects.is_empty() {
                println!("{}", "No projects found.".yellow());
                return Ok(());
            }

            println!("{}", "Projects:".bold());
            for proj in projects {
                if detail {
                    println!("\n  {} ({})", proj.name.cyan(), proj.id);
                    println!("    Created: {}", proj.created_at);
                } else {
                    println!("  - {} ({})", proj.name.cyan(), proj.id);
                }
            }
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to list projects: {}", e))
        }
    }
}

async fn show_project(config: Config, project: String) -> anyhow::Result<()> {
    utils::spinner_start("Fetching project...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.get_project(&project).await {
        Ok(proj) => {
            utils::spinner_stop();
            println!("{}", format!("Project: {}", proj.name).bold());
            println!("ID: {}", proj.id);
            println!("Created: {}", proj.created_at);
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to fetch project: {}", e))
        }
    }
}

async fn create_project(config: Config, name: String, description: Option<String>) -> anyhow::Result<()> {
    utils::spinner_start("Creating project...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.create_project(&name, description.as_deref()).await {
        Ok(proj) => {
            utils::spinner_stop();
            println!("{}", format!("✓ Project '{}' created", name).green().bold());
            println!("  ID: {}", proj.id);
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to create project: {}", e))
        }
    }
}

async fn delete_project(config: Config, project: String, force: bool) -> anyhow::Result<()> {
    if !force {
        let confirm = utils::confirm(&format!("Delete project '{}'? This cannot be undone.", project));
        if !confirm {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    utils::spinner_start("Deleting project...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.delete_project(&project).await {
        Ok(_) => {
            utils::spinner_stop();
            println!("{}", "✓ Project deleted successfully".green().bold());
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to delete project: {}", e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::utils;
use colored::*;

#[derive(Parser)]
pub struct DeployArgs {
    #[command(subcommand)]
    command: DeployCommand,
}

#[derive(Subcommand)]
enum DeployCommand {
    /// Deploy code to server
    Push {
        /// Project ID or name
        #[arg(short, long)]
        project: Option<String>,
        /// Deployment message
        #[arg(short, long)]
        message: Option<String>,
        /// Watch for changes and auto-deploy
        #[arg(short, long)]
        watch: bool,
    },
    /// Pull deployed code
    Pull {
        /// Project ID or name
        #[arg(short, long)]
        project: Option<String>,
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Sync code with server
    Sync {
        /// Project ID or name
        #[arg(short, long)]
        project: Option<String>,
        /// Direction (push/pull/both)
        #[arg(short, long, default_value = "both")]
        direction: String,
    },
    /// Analyze code before deployment
    Analyze {
        /// Project ID or name
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Show deployment history
    History {
        /// Project ID or name
        #[arg(short, long)]
        project: Option<String>,
        /// Number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

pub async fn execute(config: Config, args: DeployArgs) -> anyhow::Result<()> {
    if config.auth_token.is_empty() {
        println!("{}", "Not authenticated. Run 'cx7 auth login' first.".red());
        return Ok(());
    }

    match args.command {
        DeployCommand::Push { project, message, watch } => push(config, project, message, watch).await,
        DeployCommand::Pull { project, output } => pull(config, project, output).await,
        DeployCommand::Sync { project, direction } => sync(config, project, direction).await,
        DeployCommand::Analyze { project } => analyze(config, project).await,
        DeployCommand::History { project, limit } => history(config, project, limit).await,
    }
}

async fn push(config: Config, project: Option<String>, message: Option<String>, watch: bool) -> anyhow::Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());
    let message = message.unwrap_or_else(|| utils::prompt("Deployment message: "));

    utils::spinner_start("Collecting files...");

    let current_dir = std::env::current_dir()?;
    let files = collect_files(&current_dir)?;

    utils::spinner_stop();
    println!("{}", format!("Found {} files to deploy", files.len()).cyan());

    utils::spinner_start("Uploading...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.deploy_code(&project, &files, &message).await {
        Ok(deployment) => {
            utils::spinner_stop();
            println!("{}", "✓ Deployment successful".green().bold());
            println!("  ID: {}", deployment.id);
            println!("  Status: {}", deployment.status);
            
            if watch {
                println!("{}", "Watching for changes... (Ctrl+C to stop)".yellow());
                // Watch implementation would go here
            }
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Deployment failed: {}", e))
        }
    }
}

async fn pull(config: Config, project: Option<String>, output: Option<String>) -> anyhow::Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());
    let output_dir = output.unwrap_or_else(|| "./deployed".to_string());

    utils::spinner_start("Pulling code...");

    std::fs::create_dir_all(&output_dir)?;

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.pull_code(&project).await {
        Ok(files) => {
            utils::spinner_stop();
            
            for file in files {
                let file_path = std::path::PathBuf::from(&output_dir).join(&file.path);
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&file_path, &file.content)?;
            }

            println!("{}", "✓ Code pulled successfully".green().bold());
            println!("  Output: {}", output_dir);
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Pull failed: {}", e))
        }
    }
}

async fn sync(config: Config, project: Option<String>, direction: String) -> anyhow::Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());

    match direction.as_str() {
        "push" | "p" => push(config, Some(project), None, false).await,
        "pull" | "l" => pull(config, Some(project), None).await,
        "both" | "b" => {
            push(config.clone(), Some(project.clone()), None, false).await?;
            pull(config, Some(project), None).await
        }
        _ => {
            println!("{}", format!("Invalid direction: {}. Use 'push', 'pull', or 'both'", direction).red());
            Ok(())
        }
    }
}

async fn analyze(config: Config, project: Option<String>) -> anyhow::Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());

    utils::spinner_start("Analyzing code...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.analyze_code(&project).await {
        Ok(analysis) => {
            utils::spinner_stop();
            println!("{}", "Code Analysis:".bold());
            println!("  Lines: {}", analysis.lines_of_code);
            println!("  Complexity: {}", analysis.complexity);
            println!("  Issues: {}", analysis.issues);
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Analysis failed: {}", e))
        }
    }
}

async fn history(config: Config, project: Option<String>, limit: usize) -> anyhow::Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());

    utils::spinner_start("Fetching deployment history...");

    let client = crate::client::ApiClient::new(&config.server_url, Some(&config.auth_token));
    match client.get_deployment_history(&project, limit).await {
        Ok(deployments) => {
            utils::spinner_stop();
            println!("{}", "Deployment History:".bold());
            for (idx, deployment) in deployments.iter().enumerate() {
                println!("\n  {} ({})", deployment.id, deployment.status.cyan());
                println!("     Message: {}", deployment.message);
                println!("     Date: {}", deployment.created_at);
            }
            Ok(())
        }
        Err(e) => {
            utils::spinner_stop();
            Err(anyhow::anyhow!("Failed to fetch history: {}", e))
        }
    }
}

fn collect_files(dir: &std::path::Path) -> anyhow::Result<Vec<String>> {
    let mut files = Vec::new();
    
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if !entry.path().to_string_lossy().contains(".git")
            && !entry.path().to_string_lossy().contains(".cx7")
            && !entry.path().to_string_lossy().contains("target")
            && !entry.path().to_string_lossy().contains("node_modules")
        {
            if let Ok(rel_path) = entry.path().strip_prefix(dir) {
                files.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(files)
}

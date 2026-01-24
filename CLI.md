# CompileX7 CLI - Command Reference

The CompileX7 CLI (`cx7`) is a powerful command-line tool for local development, project management, code deployment, and agent orchestration. This guide covers all available commands and common workflows.

## Installation

### Prerequisites
- Rust 1.70+
- Cargo

### Build from Source
```bash
cd cli
cargo build --release
cargo install --path .
```

The binary will be installed as `cx7` and available globally.

## Configuration

Configuration is stored at `~/.config/compilex7/config.toml`. The CLI automatically creates this file on first use.

### Configure Server URL
```bash
cx7 config set server http://your-compilex7-server.com
```

### View Configuration
```bash
cx7 config show
```

## Authentication

All commands require authentication except `cx7 auth login`.

### Login
```bash
cx7 auth login --email your@email.com
# Enter password when prompted
```

### Check Current User
```bash
cx7 auth whoami
```

### Refresh Token
```bash
cx7 auth refresh
```

### Logout
```bash
cx7 auth logout
```

## Project Management

### Initialize Local Project
Creates a `.cx7/project.toml` file in the current directory:
```bash
cx7 project init --name "My Project"
```

### List All Projects
```bash
cx7 project list              # Brief listing
cx7 project list --detail     # Detailed view with timestamps
```

### Show Project Details
```bash
cx7 project show <project-id>
```

### Create New Project
```bash
cx7 project create --name "New Project" --description "Optional description"
```

### Delete Project
```bash
cx7 project delete <project-id>        # Confirm before deleting
cx7 project delete <project-id> --force # Skip confirmation
```

## Code Deployment

### Deploy (Push) Code
Push local code to the server:
```bash
cx7 deploy push --project "my-project" --message "Initial deployment"
cx7 deploy push --watch                 # Watch for changes and auto-deploy
```

### Pull Deployed Code
Retrieve code from the server:
```bash
cx7 deploy pull --project "my-project" --output ./pulled-code
```

### Sync Code (Bidirectional)
```bash
cx7 deploy sync --project "my-project" --direction push  # Push only
cx7 deploy sync --project "my-project" --direction pull  # Pull only
cx7 deploy sync --project "my-project" --direction both  # Two-way sync
```

### Analyze Code
Get metrics before or after deployment:
```bash
cx7 deploy analyze --project "my-project"
```

Output includes:
- Lines of code
- Cyclomatic complexity
- Number of issues detected

### View Deployment History
```bash
cx7 deploy history --project "my-project" --limit 20
```

## Agent Management

### List Available Agents
```bash
cx7 agent list
```

Agents include:
- **backend** - Backend code generation and optimization
- **frontend** - Frontend component and UI generation
- **qa** - Code quality analysis and testing

### Run an Agent
```bash
cx7 agent run backend --project "my-project"
cx7 agent run frontend --project "my-project"
cx7 agent run qa --project "my-project"
```

### Check Agent Status
```bash
cx7 agent status backend
```

## System Status

### Check Server Health
```bash
cx7 status                  # Basic health check
cx7 status --detail         # Detailed component status
```

Shows:
- Server connectivity
- Database status
- Cache status
- Running agents

## Configuration Management

### Get Configuration Value
```bash
cx7 config get server       # Get server URL
cx7 config get email        # Get configured email
cx7 config get token        # Get token (masked)
```

### Set Configuration Value
```bash
cx7 config set server http://localhost:3000
cx7 config set email your@email.com
```

### Reset to Defaults
```bash
cx7 config reset            # Confirm before resetting
cx7 config reset --force    # Skip confirmation
```

## Global Options

All commands support these global flags:

```bash
cx7 --server http://custom-server.com <command>  # Override server URL
cx7 --debug <command>                             # Enable debug output
```

Example:
```bash
cx7 --debug --server http://localhost:3000 project list
```

## Common Workflows

### First Time Setup
```bash
cx7 auth login --email your@email.com
cx7 config set server https://compilex7.yourdomain.com
cx7 project list
```

### Create and Deploy a Project
```bash
cx7 project init --name "my-app"
# ... develop code ...
cx7 deploy push --message "Feature: User authentication"
```

### Continuous Development with Auto-Deploy
```bash
cx7 deploy push --watch  # Monitor and auto-deploy on changes
```

### Full Development Cycle
```bash
# 1. Initialize
cx7 project init --name "app"

# 2. Develop locally
# ... edit code ...

# 3. Analyze before deployment
cx7 deploy analyze

# 4. Deploy
cx7 deploy push --message "Release v1.0"

# 5. Run QA agent
cx7 agent run qa

# 6. Check results
cx7 deploy history
```

## Environment Variables

Override CLI defaults with environment variables:

```bash
# Set server URL
export CX7_SERVER=http://localhost:3000

# Now cx7 commands will use this server
cx7 auth login
```

## Troubleshooting

### "Not authenticated" Error
```bash
# Login first
cx7 auth login --email your@email.com

# Verify authentication
cx7 auth whoami
```

### "Connection refused" Error
```bash
# Check server URL
cx7 config get server

# Verify server is running
cx7 status

# Update if needed
cx7 config set server http://correct-url.com
```

### "Deployment failed" Error
```bash
# Analyze code for issues
cx7 deploy analyze

# Check deployment history
cx7 deploy history --limit 5

# Review deployment logs
cx7 --debug deploy push
```

### Token Expired
```bash
# Refresh token
cx7 auth refresh

# Or login again
cx7 auth logout
cx7 auth login
```

## Performance Tips

- Use `--watch` flag for continuous development workflows
- Run `analyze` before deploying to catch issues early
- Use `--detail` flag for debugging project issues
- Enable `--debug` flag for troubleshooting CLI issues
- Cache credentials with persistent configuration

## Advanced Usage

### Batch Operations
```bash
# Deploy multiple projects
for project in $(cx7 project list | grep -oP '\(\K[^)]*'); do
  cx7 deploy push --project "$project" --message "Batch update"
done
```

### CI/CD Integration
```bash
#!/bin/bash
set -e

cx7 auth login --email "$CI_USER" # Use env var from CI
cx7 project show "$CI_PROJECT"
cx7 deploy push --message "CI deployment"
cx7 agent run qa
```

### Automated Sync with Git
```bash
# Post-commit hook
#!/bin/bash
cx7 deploy push --message "Auto-sync: $(git log -1 --pretty=%B)"
```

## Support

- Documentation: `cx7 --help`
- Command-specific help: `cx7 <command> --help`
- Issues: Submit via GitHub Issues
- Email: support@compilex7.dev

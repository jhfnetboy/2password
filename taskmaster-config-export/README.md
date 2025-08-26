# TaskMaster Configuration Export

This directory contains a complete TaskMaster configuration exported from a working project.

## Exported Files

- `config.json` - TaskMaster AI model configuration
- `mcp.json` - Claude Code MCP server configuration
- `.cursor/mcp.json` - Cursor MCP server configuration
- `CLAUDE.md` - Claude Code integration file
- `.claude/commands/` - Custom Claude Code slash commands
- `.cursor/rules/` - Cursor development rules
- `scripts/` - Development workflow helper scripts
- `.env.example` - Environment variables template

## Quick Setup for New Project

### Option 1: Automated Setup
```bash
# Copy the setup script to your new project
cp scripts/setup-taskmaster.sh /path/to/new-project/
cd /path/to/new-project/
./setup-taskmaster.sh "Project Name" "Project Description"
```

### Option 2: Manual Setup
```bash
# Install TaskMaster
npm install -g task-master-ai

# Copy configuration files
cp config.json /path/to/new-project/.taskmaster/
cp mcp.json /path/to/new-project/
cp -r .cursor /path/to/new-project/
cp -r .claude /path/to/new-project/
cp CLAUDE.md /path/to/new-project/
cp -r scripts /path/to/new-project/

# Initialize TaskMaster
cd /path/to/new-project/
task-master init --rules claude,cursor -y

# Set up models
task-master models --set-main sonnet
```

### Next Steps

1. Create your PRD at `.taskmaster/docs/prd.txt`
2. Generate tasks: `task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0`
3. Analyze complexity: `task-master analyze-complexity`
4. Expand tasks: `task-master expand --all`
5. Start development: `./scripts/tm-workflow.sh start`

## Configuration Details

### Model Configuration
- Main model: Claude Code Sonnet (no API key required)
- Research model: Perplexity Sonar Pro (requires API key)
- Fallback model: Claude 3.7 Sonnet (requires API key)

### MCP Integration
Both Claude Code and Cursor are configured with TaskMaster MCP server for seamless AI-assisted development.

### Custom Commands
- `taskmaster-next` - Find next task to work on
- `taskmaster-complete` - Mark task complete and show next
- `taskmaster-status` - Show project status

### Workflow Scripts
- `./scripts/tm-workflow.sh start` - Begin development session
- `./scripts/tm-workflow.sh next` - Show next task
- `./scripts/tm-workflow.sh done <id>` - Complete task
- `./scripts/tm-workflow.sh status` - Project overview

Export created on: Tue Aug 26 11:03:34 +07 2025

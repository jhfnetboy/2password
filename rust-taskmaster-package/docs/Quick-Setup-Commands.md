# TaskMaster Quick Setup Commands

## One-Line Setup (Copy & Paste)

For a new project, run this single command:

```bash
npm install -g task-master-ai && task-master init --rules claude,cursor --aliases -y && task-master models --set-main sonnet
```

## Step-by-Step Breakdown

```bash
# 1. Install TaskMaster globally
npm install -g task-master-ai

# 2. Initialize in your project with Claude Code integration
task-master init --rules claude,cursor --aliases --name="Your Project" --description="Your project description" -y

# 3. Configure Claude Code model (no API key needed)
task-master models --set-main sonnet

# 4. Create your PRD file
# Edit .taskmaster/docs/prd.txt with your requirements

# 5. Generate tasks from PRD
task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0

# 6. Analyze complexity and expand
task-master analyze-complexity && task-master expand --all

# 7. Start development
task-master next
```

## Copy Configuration From This Project

If you want to replicate the exact setup from this project:

```bash
# Clone or download this repository
git clone <this-repo-url>

# Copy the setup script to your new project
cp 2password/scripts/setup-taskmaster.sh /path/to/your-new-project/

# Run setup in your project directory
cd /path/to/your-new-project
./setup-taskmaster.sh "Your Project Name" "Your project description"
```

## Export Current Configuration

To export your current TaskMaster configuration for reuse:

```bash
# Run the export script
./scripts/export-taskmaster-config.sh

# This creates:
# - taskmaster-config-export/ directory with all config files
# - taskmaster-config-YYYYMMDD_HHMMSS.tar.gz archive
```

## Quick Test

Verify your setup works:

```bash
# Check TaskMaster is installed
task-master --version

# Check models are configured
task-master models

# Test with Claude Code (if installed)
# Open Claude Code and type: "Show me my current tasks"
```

## Template PRD Structure

Minimal PRD template to get started:

```markdown
# Your Project - PRD

## Product Overview
[What does your product do?]

## Core Features (Phase 1)

### Feature 1: [Name]
**Description**: [What it does]
**Requirements**:
- [Requirement 1]
- [Requirement 2]

**Acceptance Criteria**:
- [ ] [Criterion 1]
- [ ] [Criterion 2]

### Feature 2: [Name]
[Continue with more features...]

## Technical Requirements
- Technology stack
- Performance requirements  
- Security considerations

## Testing Strategy
- Unit testing approach
- Integration testing needs
- User acceptance testing
```

## Common Issues & Solutions

### MCP Not Working
```bash
# Restart Claude Code
# Check .mcp.json exists and has correct format
cat .mcp.json
```

### No Tasks Generated
```bash
# Check your PRD file exists and has content
cat .taskmaster/docs/prd.txt

# Verify model is configured
task-master models
```

### Dependencies Issues
```bash
# Fix broken dependencies
task-master fix-dependencies

# Validate all dependencies
task-master validate-dependencies
```

## Daily Workflow Commands

```bash
# Morning routine
task-master list              # See all tasks
task-master next             # Get next task to work on

# During development
task-master show 1.2         # View task details
task-master set-status --id=1.2 --status=in-progress

# End of task
task-master set-status --id=1.2 --status=done
task-master next             # See what's next
```

## Natural Language with Claude Code

Once MCP is configured, you can use natural language:

- "Show me my current tasks"
- "What should I work on next?"
- "Mark task 1.2 as completed"
- "Add a new task for user authentication"
- "Expand task 3 into subtasks"
- "Generate tasks from my PRD"
# TaskMaster Setup Guide for New Projects

This guide provides step-by-step instructions to set up TaskMaster with Claude Code integration for any new project.

## Prerequisites

- Node.js (for npm/npx)
- Git repository
- Claude Code CLI installed
- A detailed PRD (Product Requirements Document)

## Quick Setup (5 minutes)

### 1. Install TaskMaster

```bash
# Install globally
npm install -g task-master-ai
```

### 2. Initialize TaskMaster in Your Project

```bash
# Navigate to your project directory
cd your-project-name

# Initialize with Claude Code integration
task-master init --rules claude,cursor --aliases --name="Your Project Name" --description="Your project description" --version="0.1.0" --author="Your Name" -y
```

### 3. Configure Models

```bash
# Set up Claude Code model (no API key required)
task-master models --set-main sonnet

# Verify configuration
task-master models
```

### 4. Prepare Your PRD

```bash
# Copy your PRD to the TaskMaster docs directory
cp your-prd.md .taskmaster/docs/prd.txt
```

### 5. Generate Tasks from PRD

```bash
# Parse PRD and generate tasks (let AI decide optimal number)
task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0
```

### 6. Analyze Complexity and Expand Tasks

```bash
# Analyze task complexity
task-master analyze-complexity

# Expand all tasks into subtasks based on complexity
task-master expand --all
```

### 7. Start Development Workflow

```bash
# View all tasks
task-master list --with-subtasks

# See next task to work on
task-master next

# Start working on first task
task-master set-status --id=1.1 --status=in-progress
```

## Detailed Configuration Files

### Required Files Structure

After running the setup, you should have:

```
your-project/
├── .taskmaster/
│   ├── CLAUDE.md              # Auto-loaded by Claude Code
│   ├── config.json            # AI model configuration
│   ├── state.json             # Current state
│   ├── tasks/
│   │   └── tasks.json         # Generated tasks
│   ├── docs/
│   │   └── prd.txt           # Your PRD
│   ├── reports/
│   │   └── task-complexity-report.json
│   └── templates/
├── .claude/
│   └── commands/              # Custom slash commands
├── .cursor/
│   └── mcp.json              # Cursor MCP configuration
├── .mcp.json                 # Claude Code MCP configuration
├── CLAUDE.md                 # Claude Code integration
└── .env.example              # Environment template
```

## Manual Configuration (If Needed)

### MCP Configuration for Claude Code

Create `.mcp.json`:

```json
{
  "mcpServers": {
    "task-master-ai": {
      "command": "npx",
      "args": ["-y", "--package=task-master-ai", "task-master-ai"],
      "env": {
        "ANTHROPIC_API_KEY": "your_key_here",
        "PERPLEXITY_API_KEY": "your_key_here",
        "OPENAI_API_KEY": "your_key_here",
        "GOOGLE_API_KEY": "your_key_here"
      }
    }
  }
}
```

### Cursor MCP Configuration

Create `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "task-master-ai": {
      "command": "npx",
      "args": ["-y", "--package=task-master-ai", "task-master-ai"],
      "env": {
        "ANTHROPIC_API_KEY": "your_anthropic_key_here",
        "PERPLEXITY_API_KEY": "your_perplexity_key_here"
      }
    }
  }
}
```

## PRD Template

Create your `prd.txt` file following this structure:

```markdown
# Project Name - Product Requirements Document (PRD)

## Product Overview

### Product Positioning
Brief description of what your product does and who it's for.

### Target Users
- Primary users
- Use cases
- Platform requirements

## Product Goals

### Core Objectives
1. Primary goal
2. Secondary goal
3. Success metrics

## Functional Requirements

### Core Features (MVP)

#### Feature 1: [Feature Name]
**Description**: What this feature does
**Requirements**:
- Specific requirement 1
- Specific requirement 2
- Specific requirement 3

**Acceptance Criteria**:
- [ ] Criterion 1
- [ ] Criterion 2

#### Feature 2: [Feature Name]
[Continue with more features...]

## Technical Requirements

### Architecture
- Technology stack
- Performance requirements
- Security requirements

### Development Phases

#### Phase 1 (MVP) - X weeks
Core functionality description

#### Phase 2 - X weeks
Extended features description

## Testing Requirements
- Unit testing strategy
- Integration testing
- Performance testing
- Security testing
```

## Daily Development Workflow

### Morning Routine

```bash
# Check current project status
task-master list

# See what to work on next
task-master next

# Start working on a task
task-master set-status --id=X.Y --status=in-progress
```

### During Development

```bash
# View task details
task-master show X.Y

# Add implementation notes
task-master update-subtask --id=X.Y --prompt="Implementation notes and progress"

# Complete a task
task-master set-status --id=X.Y --status=done
```

### End of Day

```bash
# Review overall progress
task-master list --with-subtasks

# Check next priorities
task-master next
```

## Advanced Usage

### Adding New Tasks

```bash
# Add individual task
task-master add-task --prompt="Description of new task"

# Add task with dependencies
task-master add-task --prompt="New feature" --dependencies=1,2,3 --priority=high
```

### Managing Dependencies

```bash
# Add dependency
task-master add-dependency --id=5 --depends-on=3

# Fix broken dependencies
task-master fix-dependencies

# Validate dependencies
task-master validate-dependencies
```

### Research and Updates

```bash
# Research latest information
task-master research "What are the best practices for [your topic]?" --save-to=X.Y

# Update tasks based on new learnings
task-master update --from=5 --prompt="New requirements based on research"
```

## Claude Code Integration Commands

### Custom Slash Commands

Create `.claude/commands/taskmaster-next.md`:

```markdown
Find the next available Task Master task and show its details.

Steps:

1. Run `task-master next` to get the next task
2. If a task is available, run `task-master show <id>` for full details
3. Provide a summary of what needs to be implemented
4. Suggest the first implementation step
```

Create `.claude/commands/taskmaster-complete.md`:

```markdown
Complete a Task Master task: $ARGUMENTS

Steps:

1. Review the current task with `task-master show $ARGUMENTS`
2. Verify all implementation is complete
3. Run any tests related to this task
4. Mark as complete: `task-master set-status --id=$ARGUMENTS --status=done`
5. Show the next available task with `task-master next`
```

### Natural Language Commands in Claude Code

You can use natural language with Claude Code:

```
"Parse my PRD and generate tasks"
"Show me the next task to work on"
"Mark task 1.2 as completed and show me what's next"
"Expand task 5 into subtasks"
"Add a new task for implementing user authentication"
```

## Troubleshooting

### Common Issues

1. **MCP not working**: Restart Claude Code and check `.mcp.json` configuration
2. **Tasks not generating**: Ensure your PRD is detailed and in the correct location
3. **Dependencies broken**: Run `task-master fix-dependencies`
4. **Model not responding**: Check `task-master models` and verify Claude Code is available

### Validation Commands

```bash
# Check TaskMaster installation
task-master --version

# Verify model configuration
task-master models

# Test MCP connection (in Claude Code)
# Type: "Show me my current tasks"
```

## Export/Import Tasks

### Export Tasks to Another Project

```bash
# Export current task structure
cp .taskmaster/tasks/tasks.json ~/backup/project-tasks-$(date +%Y%m%d).json

# Export configuration
cp .taskmaster/config.json ~/backup/taskmaster-config.json
```

### Import Tasks to New Project

```bash
# After setting up TaskMaster in new project
cp ~/backup/project-tasks-YYYYMMDD.json .taskmaster/tasks/tasks.json
cp ~/backup/taskmaster-config.json .taskmaster/config.json

# Regenerate task files
task-master generate
```

## Team Collaboration

### Git Integration

Add to `.gitignore`:
```
# TaskMaster - Keep these
!.taskmaster/
!.taskmaster/tasks/tasks.json
!.taskmaster/config.json
!.taskmaster/docs/

# TaskMaster - Ignore these
.taskmaster/reports/
.taskmaster/state.json
```

### Sharing Task Progress

```bash
# Generate task summary for team updates
task-master list --status=done
task-master list --status=in-progress
```

## Tips for Success

1. **Write detailed PRDs**: The more specific your requirements, the better tasks TaskMaster generates
2. **Use complexity analysis**: Always run complexity analysis before expanding tasks
3. **Update tasks regularly**: Use `update-subtask` to log progress and learnings
4. **Review dependencies**: Use `validate-dependencies` regularly to ensure logical flow
5. **Commit task updates**: Include task progress in your git commits

## Next Steps

After setup:

1. Review generated tasks and adjust priorities if needed
2. Start with the first task (usually infrastructure setup)
3. Use TaskMaster's workflow to maintain development momentum
4. Regularly review and update task progress
5. Use the complexity reports to plan development iterations

---

This setup can be reused for any project - just replace the PRD content and project-specific details!
#!/bin/bash

# TaskMaster Configuration Export Script
# Exports current TaskMaster configuration to reuse in other projects

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

EXPORT_DIR="taskmaster-config-export"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        TaskMaster Configuration Export       ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Create export directory
mkdir -p "$EXPORT_DIR"

echo -e "${YELLOW}Exporting TaskMaster configuration...${NC}"

# Export TaskMaster configuration
if [ -f ".taskmaster/config.json" ]; then
    cp .taskmaster/config.json "$EXPORT_DIR/config.json"
    echo -e "${GREEN}✅ TaskMaster config exported${NC}"
fi

# Export MCP configurations
if [ -f ".mcp.json" ]; then
    cp .mcp.json "$EXPORT_DIR/mcp.json"
    echo -e "${GREEN}✅ Claude Code MCP config exported${NC}"
fi

if [ -f ".cursor/mcp.json" ]; then
    mkdir -p "$EXPORT_DIR/.cursor"
    cp .cursor/mcp.json "$EXPORT_DIR/.cursor/mcp.json"
    echo -e "${GREEN}✅ Cursor MCP config exported${NC}"
fi

# Export Claude Code integration files
if [ -f "CLAUDE.md" ]; then
    cp CLAUDE.md "$EXPORT_DIR/CLAUDE.md"
    echo -e "${GREEN}✅ Claude Code integration file exported${NC}"
fi

# Export custom commands
if [ -d ".claude/commands" ]; then
    cp -r .claude/commands "$EXPORT_DIR/.claude/"
    echo -e "${GREEN}✅ Custom Claude commands exported${NC}"
fi

# Export rules
if [ -d ".cursor/rules" ]; then
    cp -r .cursor/rules "$EXPORT_DIR/.cursor/"
    echo -e "${GREEN}✅ Cursor rules exported${NC}"
fi

# Export workflow scripts
if [ -d "scripts" ]; then
    cp -r scripts "$EXPORT_DIR/"
    echo -e "${GREEN}✅ Workflow scripts exported${NC}"
fi

# Export environment template
if [ -f ".env.example" ]; then
    cp .env.example "$EXPORT_DIR/.env.example"
    echo -e "${GREEN}✅ Environment template exported${NC}"
fi

# Create README for the export
cat > "$EXPORT_DIR/README.md" << EOF
# TaskMaster Configuration Export

This directory contains a complete TaskMaster configuration exported from a working project.

## Exported Files

- \`config.json\` - TaskMaster AI model configuration
- \`mcp.json\` - Claude Code MCP server configuration
- \`.cursor/mcp.json\` - Cursor MCP server configuration
- \`CLAUDE.md\` - Claude Code integration file
- \`.claude/commands/\` - Custom Claude Code slash commands
- \`.cursor/rules/\` - Cursor development rules
- \`scripts/\` - Development workflow helper scripts
- \`.env.example\` - Environment variables template

## Quick Setup for New Project

### Option 1: Automated Setup
\`\`\`bash
# Copy the setup script to your new project
cp scripts/setup-taskmaster.sh /path/to/new-project/
cd /path/to/new-project/
./setup-taskmaster.sh "Project Name" "Project Description"
\`\`\`

### Option 2: Manual Setup
\`\`\`bash
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
\`\`\`

### Next Steps

1. Create your PRD at \`.taskmaster/docs/prd.txt\`
2. Generate tasks: \`task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0\`
3. Analyze complexity: \`task-master analyze-complexity\`
4. Expand tasks: \`task-master expand --all\`
5. Start development: \`./scripts/tm-workflow.sh start\`

## Configuration Details

### Model Configuration
- Main model: Claude Code Sonnet (no API key required)
- Research model: Perplexity Sonar Pro (requires API key)
- Fallback model: Claude 3.7 Sonnet (requires API key)

### MCP Integration
Both Claude Code and Cursor are configured with TaskMaster MCP server for seamless AI-assisted development.

### Custom Commands
- \`taskmaster-next\` - Find next task to work on
- \`taskmaster-complete\` - Mark task complete and show next
- \`taskmaster-status\` - Show project status

### Workflow Scripts
- \`./scripts/tm-workflow.sh start\` - Begin development session
- \`./scripts/tm-workflow.sh next\` - Show next task
- \`./scripts/tm-workflow.sh done <id>\` - Complete task
- \`./scripts/tm-workflow.sh status\` - Project overview

Export created on: $(date)
EOF

# Create archive
tar -czf "taskmaster-config-${TIMESTAMP}.tar.gz" "$EXPORT_DIR"

echo ""
echo -e "${GREEN}✅ Configuration exported successfully!${NC}"
echo ""
echo -e "${YELLOW}Export location:${NC} $EXPORT_DIR/"
echo -e "${YELLOW}Archive created:${NC} taskmaster-config-${TIMESTAMP}.tar.gz"
echo ""
echo -e "${BLUE}To use in a new project:${NC}"
echo "1. Extract: tar -xzf taskmaster-config-${TIMESTAMP}.tar.gz"
echo "2. Copy files to new project directory"
echo "3. Run: ./scripts/setup-taskmaster.sh"
echo ""
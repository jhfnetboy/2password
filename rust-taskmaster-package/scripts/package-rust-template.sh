#!/bin/bash

# Package Rust TaskMaster Template for Reuse
# Creates a complete package with all Rust development automation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PACKAGE_DIR="rust-taskmaster-package"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        Rust TaskMaster Package Creation     ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Clean up old package if exists
rm -rf "$PACKAGE_DIR"
rm -f rust-taskmaster-*.tar.gz

# Create package directory
mkdir -p "$PACKAGE_DIR"

echo -e "${YELLOW}Packaging Rust TaskMaster template...${NC}"

# Copy core TaskMaster files
if [ -f ".taskmaster/config.json" ]; then
    mkdir -p "$PACKAGE_DIR/.taskmaster"
    cp .taskmaster/config.json "$PACKAGE_DIR/.taskmaster/"
    echo -e "${GREEN}✅ TaskMaster config copied${NC}"
fi

# Copy MCP configurations
if [ -f ".mcp.json" ]; then
    cp .mcp.json "$PACKAGE_DIR/"
    echo -e "${GREEN}✅ Claude Code MCP config copied${NC}"
fi

if [ -f ".cursor/mcp.json" ]; then
    mkdir -p "$PACKAGE_DIR/.cursor"
    cp .cursor/mcp.json "$PACKAGE_DIR/.cursor/"
    echo -e "${GREEN}✅ Cursor MCP config copied${NC}"
fi

# Copy Claude Code integration
if [ -f "CLAUDE.md" ]; then
    cp CLAUDE.md "$PACKAGE_DIR/"
    echo -e "${GREEN}✅ Claude integration copied${NC}"
fi

# Copy custom commands (both general and Rust-specific)
if [ -d ".claude/commands" ]; then
    cp -r .claude "$PACKAGE_DIR/"
    echo -e "${GREEN}✅ Custom Claude commands copied${NC}"
fi

# Copy Cursor rules
if [ -d ".cursor/rules" ]; then
    mkdir -p "$PACKAGE_DIR/.cursor"
    cp -r .cursor/rules "$PACKAGE_DIR/.cursor/"
    echo -e "${GREEN}✅ Cursor rules copied${NC}"
fi

# Copy all scripts
cp -r scripts "$PACKAGE_DIR/"
echo -e "${GREEN}✅ All scripts copied${NC}"

# Copy templates
cp -r templates "$PACKAGE_DIR/"
echo -e "${GREEN}✅ Templates copied${NC}"

# Copy documentation
mkdir -p "$PACKAGE_DIR/docs"
cp docs/TaskMaster-Setup-Guide.md "$PACKAGE_DIR/docs/" 2>/dev/null || true
cp docs/Quick-Setup-Commands.md "$PACKAGE_DIR/docs/" 2>/dev/null || true
cp docs/Rust-Cache-Management.md "$PACKAGE_DIR/docs/" 2>/dev/null || true
echo -e "${GREEN}✅ Documentation copied${NC}"

# Copy Cargo configuration template
if [ -f ".cargo/config.toml" ]; then
    cp -r .cargo "$PACKAGE_DIR/"
    echo -e "${GREEN}✅ Cargo config copied${NC}"
fi

# Create environment template
cat > "$PACKAGE_DIR/.env.example" << 'EOF'
# TaskMaster Environment Variables
# Copy this to .env and fill in your API keys

# Optional: Anthropic API Key for advanced models
ANTHROPIC_API_KEY=your_anthropic_key_here

# Optional: Perplexity API Key for research
PERPLEXITY_API_KEY=your_perplexity_key_here

# Optional: OpenAI API Key
OPENAI_API_KEY=your_openai_key_here

# Optional: Google API Key
GOOGLE_API_KEY=your_google_key_here

# Rust-specific settings
RUST_LOG=info
RUST_BACKTRACE=1

# Build optimization
CARGO_INCREMENTAL=1
EOF

echo -e "${GREEN}✅ Environment template created${NC}"

# Create master setup script for new projects
cat > "$PACKAGE_DIR/setup-new-rust-project.sh" << 'EOF'
#!/bin/bash

# Master Setup Script for New Rust Projects with TaskMaster
# Usage: ./setup-new-rust-project.sh [project-name] [project-description]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
PROJECT_NAME="${1:-$(basename "$(pwd)")}"
PROJECT_DESC="${2:-A new Rust project with TaskMaster and automated cache management}"

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     New Rust Project with TaskMaster Setup  ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Project: $PROJECT_NAME${NC}"
echo -e "${YELLOW}Description: $PROJECT_DESC${NC}"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command_exists rustc; then
    echo -e "${RED}❌ Rust is not installed. Please install Rust first:${NC}"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

if ! command_exists node; then
    echo -e "${RED}❌ Node.js is not installed. Please install Node.js first.${NC}"
    exit 1
fi

if ! command_exists git; then
    echo -e "${RED}❌ Git is not installed. Please install Git first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"
echo ""

# Initialize git repository if needed
if [ ! -d ".git" ]; then
    echo -e "${YELLOW}Initializing Git repository...${NC}"
    git init
    echo -e "${GREEN}✅ Git repository initialized${NC}"
fi

# Install TaskMaster globally if not already installed
echo -e "${YELLOW}Installing TaskMaster AI...${NC}"
if ! command_exists task-master; then
    npm install -g task-master-ai
    echo -e "${GREEN}✅ TaskMaster AI installed globally${NC}"
else
    echo -e "${GREEN}✅ TaskMaster AI already installed${NC}"
fi

# Install rskiller for Rust cache management
echo -e "${YELLOW}Installing rskiller for cache management...${NC}"
if ! command_exists rskiller; then
    cargo install rskiller
    echo -e "${GREEN}✅ rskiller installed${NC}"
else
    echo -e "${GREEN}✅ rskiller already installed${NC}"
fi

# Initialize TaskMaster
echo -e "${YELLOW}Initializing TaskMaster...${NC}"
task-master init \
    --rules claude,cursor \
    --aliases \
    --name="$PROJECT_NAME" \
    --description="$PROJECT_DESC" \
    -y

echo -e "${GREEN}✅ TaskMaster initialized${NC}"

# Configure models
echo -e "${YELLOW}Configuring AI models...${NC}"
task-master models --set-main sonnet
echo -e "${GREEN}✅ Models configured${NC}"

# Copy configuration files
echo -e "${YELLOW}Copying configuration files...${NC}"

# TaskMaster config
if [ -f ".taskmaster/config.json" ]; then
    cp .taskmaster/config.json .taskmaster/config.json.backup 2>/dev/null || true
fi

# MCP configurations
cp .mcp.json . 2>/dev/null || true
mkdir -p .cursor && cp .cursor/mcp.json .cursor/ 2>/dev/null || true

# Claude integration
cp CLAUDE.md . 2>/dev/null || true

# Custom commands
cp -r .claude . 2>/dev/null || true

# Cursor rules
mkdir -p .cursor && cp -r .cursor/rules .cursor/ 2>/dev/null || true

# Cargo configuration
mkdir -p .cargo && cp .cargo/config.toml .cargo/ 2>/dev/null || true

# Scripts
mkdir -p scripts && cp -r scripts/* scripts/ 2>/dev/null || true

# Make scripts executable
chmod +x scripts/*.sh 2>/dev/null || true

echo -e "${GREEN}✅ Configuration files copied${NC}"

# Create sample PRD for Rust project
if [ ! -f ".taskmaster/docs/prd.txt" ]; then
    echo -e "${YELLOW}Creating Rust project PRD template...${NC}"
    mkdir -p .taskmaster/docs
    cp templates/rust-project-template.md .taskmaster/docs/prd.txt
    
    # Replace placeholders
    sed -i.bak "s/\[PROJECT_NAME\]/$PROJECT_NAME/g" .taskmaster/docs/prd.txt && rm .taskmaster/docs/prd.txt.bak 2>/dev/null || true
    
    echo -e "${GREEN}✅ Rust PRD template created${NC}"
fi

# Setup Rust automation
echo -e "${YELLOW}Setting up Rust automation...${NC}"
if [ -f "scripts/setup-rust-automation.sh" ]; then
    ./scripts/setup-rust-automation.sh
    echo -e "${GREEN}✅ Rust automation configured${NC}"
fi

# Initialize Cargo project if needed
if [ ! -f "Cargo.toml" ]; then
    echo -e "${YELLOW}Initializing Cargo project...${NC}"
    cargo init --name "$PROJECT_NAME"
    echo -e "${GREEN}✅ Cargo project initialized${NC}"
fi

# Copy environment template
cp .env.example . 2>/dev/null || true

echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║            Setup Complete! 🎉                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Next Steps:${NC}"
echo ""
echo -e "${YELLOW}1. Edit your PRD:${NC}"
echo "   edit .taskmaster/docs/prd.txt"
echo ""
echo -e "${YELLOW}2. Generate tasks:${NC}"
echo "   task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0"
echo ""
echo -e "${YELLOW}3. Analyze and expand:${NC}"
echo "   task-master analyze-complexity && task-master expand --all"
echo ""
echo -e "${YELLOW}4. Start development:${NC}"
echo "   task-master next"
echo ""
echo -e "${YELLOW}5. Automated features:${NC}"
echo "   • Cache cleaning every 2 hours ✅"
echo "   • Optimized Cargo configuration ✅"
echo "   • Custom Claude Code commands ✅"
echo ""
echo -e "${GREEN}Happy Rust development with TaskMaster! 🦀${NC}"
EOF

chmod +x "$PACKAGE_DIR/setup-new-rust-project.sh"
echo -e "${GREEN}✅ Master setup script created${NC}"

# Create detailed README
cat > "$PACKAGE_DIR/README.md" << EOF
# Rust TaskMaster Development Package

Complete development environment package for Rust projects with TaskMaster AI, automated cache management, and Claude Code integration.

## What's Included

### 🤖 TaskMaster AI Integration
- Complete configuration for Claude Code and Cursor
- Custom slash commands for Rust development
- MCP (Model Control Protocol) setup
- Project management and task generation from PRDs

### 🦀 Rust-Specific Optimizations
- **rskiller** cache management (runs every 2 hours)
- Optimized Cargo configuration for fast builds
- Cross-platform build settings
- Development workflow scripts

### 📝 Templates and Documentation
- Comprehensive Rust PRD template
- Project setup guides
- Cache management documentation
- Development workflow guides

### 🛠️ Development Tools
- Automated cache cleaning scripts
- Build optimization scripts
- Claude Code custom commands
- CI/CD templates

## Quick Start

### Option 1: Automated Setup
\`\`\`bash
# Extract this package to your new project
tar -xzf rust-taskmaster-package.tar.gz
cd your-new-project/
cp -r rust-taskmaster-package/* .

# Run the master setup script
./setup-new-rust-project.sh "Your Project Name" "Your description"
\`\`\`

### Option 2: Manual Setup
\`\`\`bash
# Install prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
npm install -g task-master-ai
cargo install rskiller

# Copy files and run setup
cp -r rust-taskmaster-package/* your-project/
cd your-project/
./setup-new-rust-project.sh
\`\`\`

## Features

### Automated Cache Management
- **rskiller** cleans Rust caches every 2 hours
- Non-interactive safe cleaning mode
- Detailed logging and monitoring
- Manual control scripts

### TaskMaster Integration  
- Generate development tasks from PRDs
- AI-powered complexity analysis
- Task dependency management
- Progress tracking and reporting

### Claude Code Commands
- \`/rust-build\` - Build with comprehensive checks
- \`/rust-clean\` - Clean caches and artifacts  
- \`/rust-optimize\` - Performance and size optimization
- \`/taskmaster-next\` - Get next task to work on
- \`/taskmaster-complete\` - Mark tasks as done

### Development Workflow
\`\`\`bash
# Daily routine
./scripts/rust-workflow.sh dev      # Start dev session
./scripts/rust-workflow.sh build    # Build project
./scripts/rust-workflow.sh test     # Run all tests
./scripts/rust-workflow.sh clean    # Clean caches
\`\`\`

## File Structure

\`\`\`
rust-taskmaster-package/
├── README.md                           # This file
├── setup-new-rust-project.sh          # Master setup script
├── .taskmaster/config.json             # TaskMaster configuration
├── .mcp.json                          # Claude Code MCP config
├── .cursor/mcp.json                   # Cursor MCP config
├── CLAUDE.md                          # Claude integration
├── .claude/commands/                  # Custom slash commands
├── .cargo/config.toml                 # Cargo optimization
├── scripts/
│   ├── setup-taskmaster.sh           # TaskMaster setup
│   ├── setup-rust-automation.sh      # Rust automation setup
│   ├── rust-clean.sh                 # Cache cleaning script
│   └── rust-workflow.sh              # Development workflow
├── templates/
│   └── rust-project-template.md      # Rust PRD template
├── docs/
│   ├── TaskMaster-Setup-Guide.md     # Setup documentation
│   ├── Quick-Setup-Commands.md       # Command reference
│   └── Rust-Cache-Management.md      # Cache management guide
└── .env.example                      # Environment template
\`\`\`

## Configuration Details

### TaskMaster Models
- **Main Model**: Claude Code Sonnet (no API key required)
- **Research Model**: Perplexity Sonar Pro (optional)
- **Fallback**: Claude 3.7 Sonnet (optional)

### Cache Management
- **Schedule**: Every 2 hours via launchd (macOS) or cron (Linux)
- **Mode**: Safe cleaning (preserves recent builds)
- **Logging**: \`~/.rust-cache-clean.log\`

### Build Optimization
- **Development**: Fast incremental builds
- **Release**: Size and performance optimized
- **Production**: Maximum optimization with LTO

## Troubleshooting

### Common Issues

1. **TaskMaster not found**
   \`\`\`bash
   npm install -g task-master-ai
   \`\`\`

2. **rskiller not installed**  
   \`\`\`bash
   cargo install rskiller
   \`\`\`

3. **MCP not working**
   - Restart Claude Code
   - Check \`.mcp.json\` configuration

4. **Automated cleaning not running**
   \`\`\`bash
   # macOS
   launchctl list | grep rust-cache
   
   # Linux  
   crontab -l | grep rskiller
   \`\`\`

### Validation Commands

\`\`\`bash
# Check installations
rustc --version
cargo --version  
task-master --version
rskiller --version

# Test configurations
task-master models
task-master list
./scripts/rust-clean.sh --analyze
\`\`\`

## Support

For issues and questions:
1. Check the documentation in \`docs/\`
2. Run validation commands
3. Review log files for automated processes
4. Consult TaskMaster documentation

## Version

Package created on: $(date)
Compatible with:
- Rust 1.70+
- TaskMaster AI latest
- rskiller 0.1.0+
- Claude Code latest

---

Happy Rust development! 🦀✨
EOF

echo -e "${GREEN}✅ Documentation created${NC}"

# Create archive
echo -e "${YELLOW}Creating package archive...${NC}"
tar -czf "rust-taskmaster-package-${TIMESTAMP}.tar.gz" "$PACKAGE_DIR"

echo ""
echo -e "${GREEN}✅ Rust TaskMaster package created successfully!${NC}"
echo ""
echo -e "${YELLOW}Package contents:${NC}"
echo "  📁 Directory: $PACKAGE_DIR/"
echo "  📦 Archive: rust-taskmaster-package-${TIMESTAMP}.tar.gz"
echo ""
echo -e "${BLUE}To use in a new project:${NC}"
echo "  1. Extract: tar -xzf rust-taskmaster-package-${TIMESTAMP}.tar.gz"
echo "  2. Copy to project: cp -r rust-taskmaster-package/* your-project/"
echo "  3. Setup: ./setup-new-rust-project.sh \"Project Name\""
echo ""
echo -e "${GREEN}🎉 Complete Rust development environment ready for reuse!${NC}"
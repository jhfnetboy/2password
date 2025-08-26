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

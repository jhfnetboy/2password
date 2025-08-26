#!/bin/bash

# Rust Development Automation Setup
# Sets up automated cache cleaning and development workflow

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        Rust Development Automation Setup    ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command_exists cargo; then
    echo -e "${RED}❌ Rust/Cargo is not installed. Please install Rust first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Rust/Cargo found${NC}"

# Install rskiller if not present
if ! command_exists rskiller; then
    echo -e "${YELLOW}Installing rskiller for cache management...${NC}"
    cargo install rskiller
    echo -e "${GREEN}✅ rskiller installed${NC}"
else
    echo -e "${GREEN}✅ rskiller already installed${NC}"
fi

echo ""

# Detect OS for scheduling setup
OS_TYPE=$(uname -s)
echo -e "${YELLOW}Setting up automated cache cleaning for $OS_TYPE...${NC}"

if [[ "$OS_TYPE" == "Darwin" ]]; then
    # macOS - launchd setup
    LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
    PLIST_FILE="$LAUNCH_AGENTS_DIR/com.user.rust-cache-clean.plist"
    
    mkdir -p "$LAUNCH_AGENTS_DIR"
    
    cat > "$PLIST_FILE" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.user.rust-cache-clean</string>
    <key>ProgramArguments</key>
    <array>
        <string>$HOME/.cargo/bin/rskiller</string>
        <string>--clean</string>
        <string>--safe</string>
        <string>--no-confirm</string>
    </array>
    <key>StartInterval</key>
    <integer>7200</integer>
    <key>RunAtLoad</key>
    <false/>
    <key>StandardOutPath</key>
    <string>$HOME/.rust-cache-clean.log</string>
    <key>StandardErrorPath</key>
    <string>$HOME/.rust-cache-clean.error.log</string>
</dict>
</plist>
EOF
    
    # Load the launch agent
    launchctl unload "$PLIST_FILE" 2>/dev/null || true
    launchctl load "$PLIST_FILE"
    
    echo -e "${GREEN}✅ macOS launchd service configured${NC}"
    echo -e "${BLUE}   Cache cleaning every 2 hours${NC}"
    echo -e "${BLUE}   Logs: ~/.rust-cache-clean.log${NC}"
    
elif [[ "$OS_TYPE" == "Linux" ]]; then
    # Linux - cron setup
    CRON_COMMAND="0 */2 * * * $HOME/.cargo/bin/rskiller --clean --safe --no-confirm >> $HOME/.rust-cache-clean.log 2>&1"
    
    # Add to crontab if not already present
    if ! crontab -l 2>/dev/null | grep -q "rskiller --clean"; then
        (crontab -l 2>/dev/null; echo "$CRON_COMMAND") | crontab -
        echo -e "${GREEN}✅ Cron job configured${NC}"
        echo -e "${BLUE}   Cache cleaning every 2 hours${NC}"
    else
        echo -e "${YELLOW}⚠️  Cron job already exists${NC}"
    fi
    
else
    echo -e "${YELLOW}⚠️  Unsupported OS: $OS_TYPE${NC}"
    echo -e "${YELLOW}   You can manually run: ./scripts/rust-clean.sh --safe${NC}"
fi

echo ""

# Create development workflow integration
echo -e "${YELLOW}Setting up development workflow integration...${NC}"

# Create rust-specific TaskMaster commands
mkdir -p .claude/commands

cat > .claude/commands/rust-clean.md << 'EOF'
Clean Rust caches and build artifacts using rskiller: $ARGUMENTS

Steps:

1. Run the appropriate cleaning mode:
   - For safe cleaning: `./scripts/rust-clean.sh --safe`
   - For aggressive cleaning: `./scripts/rust-clean.sh --aggressive`
   - For complete cleaning: `./scripts/rust-clean.sh --all`
   - For analysis only: `./scripts/rust-clean.sh --analyze`

2. Show cache size analysis before and after cleaning
3. Provide recommendations for build cache management
4. Log the operation for future reference
EOF

cat > .claude/commands/rust-optimize.md << 'EOF'
Optimize Rust development environment and performance.

Steps:

1. Run cache analysis: `./scripts/rust-clean.sh --analyze`
2. Check for unnecessary dependencies in Cargo.toml
3. Verify build profiles are optimized
4. Clean old artifacts: `./scripts/rust-clean.sh --safe`
5. Suggest performance improvements for build times
EOF

echo -e "${GREEN}✅ Development workflow commands created${NC}"

# Create cargo configuration for optimization
mkdir -p .cargo

cat > .cargo/config.toml << 'EOF'
# Rust optimization configuration for development

[build]
# Use faster linker on macOS
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Use faster linker on Linux
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Optimize for development speed
[profile.dev]
opt-level = 0
debug = 2
debug-assertions = true
overflow-checks = true
lto = false
panic = 'unwind'
incremental = true
codegen-units = 256

# Optimize dependencies even in debug mode
[profile.dev.package."*"]
opt-level = 3

# Fast release builds
[profile.release]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
lto = "thin"
panic = 'abort'
incremental = false
codegen-units = 1

# Profile for CI/production builds
[profile.production]
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"
EOF

echo -e "${GREEN}✅ Cargo optimization config created${NC}"

# Create shell completion if available
if command_exists rskiller; then
    echo -e "${YELLOW}Testing rskiller installation...${NC}"
    rskiller --help > /dev/null 2>&1 && echo -e "${GREEN}✅ rskiller working correctly${NC}" || echo -e "${RED}⚠️  rskiller installation needs verification${NC}"
fi

echo ""

# Final instructions
echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║            Setup Complete! 🎉                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Rust Development Automation Configured:${NC}"
echo ""
echo -e "${YELLOW}Automated Features:${NC}"
echo "  ✅ Cache cleaning every 2 hours"
echo "  ✅ Optimized Cargo configuration"
echo "  ✅ Custom Claude Code commands"
echo "  ✅ Development workflow scripts"
echo ""
echo -e "${YELLOW}Manual Commands:${NC}"
echo "  ./scripts/rust-clean.sh --safe      # Safe cache cleaning"
echo "  ./scripts/rust-clean.sh --analyze   # Check cache sizes"
echo "  ./scripts/rust-clean.sh --all       # Complete clean"
echo ""
echo -e "${YELLOW}Claude Code Commands:${NC}"
echo "  /rust-clean --safe                  # Clean caches safely"
echo "  /rust-optimize                      # Optimize environment"
echo ""
echo -e "${YELLOW}Monitoring:${NC}"
echo "  tail -f ~/.rust-cache-clean.log     # View automation logs"
echo ""
echo -e "${GREEN}Happy Rust development! 🦀${NC}"
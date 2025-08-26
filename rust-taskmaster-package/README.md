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
```bash
# Extract this package to your new project
tar -xzf rust-taskmaster-package.tar.gz
cd your-new-project/
cp -r rust-taskmaster-package/* .

# Run the master setup script
./setup-new-rust-project.sh "Your Project Name" "Your description"
```

### Option 2: Manual Setup
```bash
# Install prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
npm install -g task-master-ai
cargo install rskiller

# Copy files and run setup
cp -r rust-taskmaster-package/* your-project/
cd your-project/
./setup-new-rust-project.sh
```

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
- `/rust-build` - Build with comprehensive checks
- `/rust-clean` - Clean caches and artifacts  
- `/rust-optimize` - Performance and size optimization
- `/taskmaster-next` - Get next task to work on
- `/taskmaster-complete` - Mark tasks as done

### Development Workflow
```bash
# Daily routine
./scripts/rust-workflow.sh dev      # Start dev session
./scripts/rust-workflow.sh build    # Build project
./scripts/rust-workflow.sh test     # Run all tests
./scripts/rust-workflow.sh clean    # Clean caches
```

## File Structure

```
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
```

## Configuration Details

### TaskMaster Models
- **Main Model**: Claude Code Sonnet (no API key required)
- **Research Model**: Perplexity Sonar Pro (optional)
- **Fallback**: Claude 3.7 Sonnet (optional)

### Cache Management
- **Schedule**: Every 2 hours via launchd (macOS) or cron (Linux)
- **Mode**: Safe cleaning (preserves recent builds)
- **Logging**: `~/.rust-cache-clean.log`

### Build Optimization
- **Development**: Fast incremental builds
- **Release**: Size and performance optimized
- **Production**: Maximum optimization with LTO

## Troubleshooting

### Common Issues

1. **TaskMaster not found**
   ```bash
   npm install -g task-master-ai
   ```

2. **rskiller not installed**  
   ```bash
   cargo install rskiller
   ```

3. **MCP not working**
   - Restart Claude Code
   - Check `.mcp.json` configuration

4. **Automated cleaning not running**
   ```bash
   # macOS
   launchctl list | grep rust-cache
   
   # Linux  
   crontab -l | grep rskiller
   ```

### Validation Commands

```bash
# Check installations
rustc --version
cargo --version  
task-master --version
rskiller --version

# Test configurations
task-master models
task-master list
./scripts/rust-clean.sh --analyze
```

## Support

For issues and questions:
1. Check the documentation in `docs/`
2. Run validation commands
3. Review log files for automated processes
4. Consult TaskMaster documentation

## Version

Package created on: Tue Aug 26 11:16:07 +07 2025
Compatible with:
- Rust 1.70+
- TaskMaster AI latest
- rskiller 0.1.0+
- Claude Code latest

---

Happy Rust development! 🦀✨

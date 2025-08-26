# Rust Cache Management with rskiller

## Overview

Automated Rust cache management using rskiller to clean build artifacts and caches every 2 hours, preventing disk space bloat during development.

## rskiller Configuration

### Non-Interactive Cleaning Commands

```bash
# Clean all Rust caches (non-interactive)
rskiller --clean --all --no-confirm

# Clean specific directories
rskiller --clean --target --no-confirm          # target/ directories
rskiller --clean --registry --no-confirm        # ~/.cargo/registry
rskiller --clean --git --no-confirm            # ~/.cargo/git

# Safe clean (keeps recent builds)
rskiller --clean --safe --no-confirm

# Aggressive clean (removes everything)
rskiller --clean --aggressive --no-confirm
```

### Recommended Clean Strategy

For development environments, use **safe mode** to preserve recent builds while cleaning old artifacts:

```bash
rskiller --clean --safe --no-confirm
```

This removes:
- Build artifacts older than 1 week
- Unused registry cache entries
- Old git repository caches
- Temporary build files

## Automated Scheduling

### macOS (launchd)

Create scheduled task that runs every 2 hours:

```xml
<!-- ~/Library/LaunchAgents/com.user.rust-cache-clean.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.user.rust-cache-clean</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/$(whoami)/.cargo/bin/rskiller</string>
        <string>--clean</string>
        <string>--safe</string>
        <string>--no-confirm</string>
    </array>
    <key>StartInterval</key>
    <integer>7200</integer>
    <key>RunAtLoad</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/tmp/rust-cache-clean.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/rust-cache-clean.error.log</string>
</dict>
</plist>
```

### Linux (cron)

Add to crontab for 2-hour intervals:

```bash
# Clean Rust cache every 2 hours
0 */2 * * * ~/.cargo/bin/rskiller --clean --safe --no-confirm >> ~/rust-cache-clean.log 2>&1
```

## Manual Management

### Daily Development Workflow

```bash
# Morning clean (aggressive)
./scripts/rust-clean.sh --aggressive

# During development (safe clean when disk space low)
./scripts/rust-clean.sh --safe

# End of day (clean everything)
./scripts/rust-clean.sh --all
```

### Monitoring Cache Size

```bash
# Check current cache sizes
rskiller --analyze

# Show detailed cache breakdown
rskiller --analyze --verbose
```

## Integration with TaskMaster

rskiller commands are integrated into TaskMaster workflow:

- Automatic cache cleaning during task transitions
- Pre-build cache analysis
- Post-completion cleanup
- Development environment maintenance

## Troubleshooting

### Common Issues

1. **Permission denied**: Ensure rskiller is installed with correct permissions
2. **Command not found**: Check `~/.cargo/bin` is in PATH
3. **Disk space still full**: Use `--aggressive` mode for thorough cleaning

### Recovery

If cache cleaning breaks builds:

```bash
# Rebuild registry cache
cargo fetch

# Rebuild specific project
cargo clean && cargo build
```

## Best Practices

1. **Use safe mode by default** - Preserves recent work
2. **Monitor disk usage** - Run `rskiller --analyze` weekly
3. **Backup important builds** - Before aggressive cleaning
4. **Schedule during low activity** - Avoid interrupting builds
5. **Log all operations** - Keep audit trail of cleanings
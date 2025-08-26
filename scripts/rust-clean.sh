#!/bin/bash

# Rust Cache Management Script with rskiller
# Usage: ./scripts/rust-clean.sh [--safe|--aggressive|--all|--analyze]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default mode
MODE="safe"
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --safe)
            MODE="safe"
            shift
            ;;
        --aggressive)
            MODE="aggressive"
            shift
            ;;
        --all)
            MODE="all"
            shift
            ;;
        --analyze)
            MODE="analyze"
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Rust Cache Management Script"
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --safe        Clean old artifacts, keep recent builds (default)"
            echo "  --aggressive  Remove all build artifacts and caches"
            echo "  --all         Clean everything including registry and git caches"
            echo "  --analyze     Analyze cache sizes without cleaning"
            echo "  --verbose     Show detailed output"
            echo "  --help        Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Rust Cache Management              ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if rskiller is installed
if ! command_exists rskiller; then
    echo -e "${RED}❌ rskiller is not installed${NC}"
    echo -e "${YELLOW}Installing rskiller...${NC}"
    cargo install rskiller
    echo -e "${GREEN}✅ rskiller installed${NC}"
    echo ""
fi

# Function to format bytes
format_bytes() {
    local bytes=$1
    if [[ $bytes -gt 1073741824 ]]; then
        echo "$(( bytes / 1073741824 )) GB"
    elif [[ $bytes -gt 1048576 ]]; then
        echo "$(( bytes / 1048576 )) MB"
    elif [[ $bytes -gt 1024 ]]; then
        echo "$(( bytes / 1024 )) KB"
    else
        echo "$bytes bytes"
    fi
}

# Function to show cache analysis
show_analysis() {
    echo -e "${YELLOW}Analyzing Rust cache sizes...${NC}"
    
    if $VERBOSE; then
        rskiller --analyze --verbose
    else
        rskiller --analyze
    fi
    
    echo ""
    echo -e "${BLUE}Cache locations:${NC}"
    echo "  ~/.cargo/registry  (downloaded crates)"
    echo "  ~/.cargo/git       (git dependencies)"
    echo "  ./target           (build artifacts)"
    echo ""
}

# Function to perform cleaning
perform_clean() {
    local clean_mode=$1
    local before_size
    local after_size
    
    echo -e "${YELLOW}Cleaning Rust caches in $clean_mode mode...${NC}"
    
    # Get size before cleaning (if analyze command works)
    if command_exists du; then
        before_size=$(du -sh ~/.cargo 2>/dev/null | cut -f1 || echo "unknown")
    else
        before_size="unknown"
    fi
    
    case $clean_mode in
        "safe")
            echo -e "${GREEN}Running safe clean (keeps recent builds)...${NC}"
            if $VERBOSE; then
                rskiller --clean --safe --verbose
            else
                rskiller --clean --safe --no-confirm
            fi
            ;;
        "aggressive")
            echo -e "${YELLOW}Running aggressive clean (removes all build artifacts)...${NC}"
            if $VERBOSE; then
                rskiller --clean --aggressive --verbose
            else
                rskiller --clean --aggressive --no-confirm
            fi
            ;;
        "all")
            echo -e "${RED}Running complete clean (removes everything)...${NC}"
            if $VERBOSE; then
                rskiller --clean --all --verbose
            else
                rskiller --clean --all --no-confirm
            fi
            ;;
    esac
    
    echo -e "${GREEN}✅ Cache cleaning completed${NC}"
    
    # Show size after cleaning
    if command_exists du && [[ "$before_size" != "unknown" ]]; then
        after_size=$(du -sh ~/.cargo 2>/dev/null | cut -f1 || echo "unknown")
        echo ""
        echo -e "${BLUE}Results:${NC}"
        echo -e "  Before: $before_size"
        echo -e "  After:  $after_size"
    fi
}

# Function to log operation
log_operation() {
    local operation=$1
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local log_file="$HOME/.rust-cache-clean.log"
    
    echo "[$timestamp] Rust cache $operation completed" >> "$log_file"
}

# Main execution
case $MODE in
    "analyze")
        show_analysis
        ;;
    "safe"|"aggressive"|"all")
        show_analysis
        echo ""
        perform_clean "$MODE"
        log_operation "$MODE clean"
        ;;
esac

echo ""
echo -e "${GREEN}🎉 Rust cache management completed!${NC}"
echo ""

# Show helpful commands
if [[ $MODE != "analyze" ]]; then
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  • Run 'cargo fetch' to rebuild registry if needed"
    echo "  • Run 'cargo build' in your project to rebuild artifacts"
    echo "  • Use './scripts/rust-clean.sh --analyze' to monitor cache sizes"
fi
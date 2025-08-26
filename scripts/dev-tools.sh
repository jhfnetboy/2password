#!/bin/bash

# Development tools script for TwoPassword
# Usage: ./scripts/dev-tools.sh [command]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

case "$1" in
    "format"|"fmt")
        echo -e "${YELLOW}🎨 Formatting code with rustfmt...${NC}"
        cargo fmt --all
        echo -e "${GREEN}✅ Code formatting complete${NC}"
        ;;
    
    "lint"|"clippy")
        echo -e "${YELLOW}🔍 Running clippy lints...${NC}"
        cargo clippy --all-targets --all-features -- -D warnings
        echo -e "${GREEN}✅ Clippy lints passed${NC}"
        ;;
    
    "test")
        echo -e "${YELLOW}🧪 Running tests...${NC}"
        cargo test --all-features
        echo -e "${GREEN}✅ All tests passed${NC}"
        ;;
    
    "check")
        echo -e "${YELLOW}⚙️  Type checking...${NC}"
        cargo check --all-targets --all-features
        echo -e "${GREEN}✅ Type check passed${NC}"
        ;;
    
    "bench")
        echo -e "${YELLOW}📊 Running benchmarks...${NC}"
        cargo bench
        echo -e "${GREEN}✅ Benchmarks complete${NC}"
        echo -e "${BLUE}📈 Results saved to target/criterion/report/index.html${NC}"
        ;;
    
    "doc")
        echo -e "${YELLOW}📚 Generating documentation...${NC}"
        cargo doc --no-deps --all-features --open
        echo -e "${GREEN}✅ Documentation generated${NC}"
        ;;
    
    "clean")
        echo -e "${YELLOW}🧹 Cleaning build artifacts...${NC}"
        cargo clean
        ./scripts/rust-clean.sh --safe
        echo -e "${GREEN}✅ Cleanup complete${NC}"
        ;;
    
    "audit")
        echo -e "${YELLOW}🔒 Security audit...${NC}"
        if command -v cargo-audit >/dev/null 2>&1; then
            cargo audit
            echo -e "${GREEN}✅ Security audit complete${NC}"
        else
            echo -e "${YELLOW}⚠️  cargo-audit not installed. Install with: cargo install cargo-audit${NC}"
        fi
        ;;
    
    "outdated")
        echo -e "${YELLOW}📦 Checking for outdated dependencies...${NC}"
        if command -v cargo-outdated >/dev/null 2>&1; then
            cargo outdated
            echo -e "${GREEN}✅ Dependency check complete${NC}"
        else
            echo -e "${YELLOW}⚠️  cargo-outdated not installed. Install with: cargo install cargo-outdated${NC}"
        fi
        ;;
    
    "coverage")
        echo -e "${YELLOW}📋 Running test coverage...${NC}"
        if command -v cargo-tarpaulin >/dev/null 2>&1; then
            cargo tarpaulin --all-features --out Html --output-dir target/coverage
            echo -e "${GREEN}✅ Coverage report generated at target/coverage/tarpaulin-report.html${NC}"
        else
            echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin${NC}"
        fi
        ;;
    
    "pre-commit")
        echo -e "${YELLOW}🚀 Running pre-commit checks...${NC}"
        echo -e "${BLUE}1/4 Formatting...${NC}"
        cargo fmt --all -- --check
        echo -e "${BLUE}2/4 Clippy lints...${NC}"
        cargo clippy --all-targets --all-features -- -D warnings
        echo -e "${BLUE}3/4 Type checking...${NC}"
        cargo check --all-targets --all-features
        echo -e "${BLUE}4/4 Running tests...${NC}"
        cargo test --all-features
        echo -e "${GREEN}✅ All pre-commit checks passed!${NC}"
        ;;
    
    "full")
        echo -e "${YELLOW}🎯 Running full development check...${NC}"
        ./scripts/dev-tools.sh format
        ./scripts/dev-tools.sh lint
        ./scripts/dev-tools.sh test
        ./scripts/dev-tools.sh doc
        echo -e "${GREEN}🎉 Full development check complete!${NC}"
        ;;
    
    "setup")
        echo -e "${YELLOW}⚙️  Setting up development tools...${NC}"
        echo -e "${BLUE}Installing recommended tools...${NC}"
        cargo install cargo-audit cargo-outdated cargo-tarpaulin
        echo -e "${GREEN}✅ Development tools installed${NC}"
        ;;
    
    *)
        echo -e "${BLUE}TwoPassword Development Tools${NC}"
        echo ""
        echo -e "${YELLOW}Usage:${NC} ./scripts/dev-tools.sh [command]"
        echo ""
        echo -e "${YELLOW}Available commands:${NC}"
        echo "  format     Format code with rustfmt"
        echo "  lint       Run clippy lints"
        echo "  test       Run all tests"
        echo "  check      Type check the project"
        echo "  bench      Run performance benchmarks"
        echo "  doc        Generate and open documentation"
        echo "  clean      Clean build artifacts and caches"
        echo "  audit      Run security audit (requires cargo-audit)"
        echo "  outdated   Check for outdated dependencies (requires cargo-outdated)"
        echo "  coverage   Generate test coverage report (requires cargo-tarpaulin)"
        echo "  pre-commit Run all pre-commit checks"
        echo "  full       Run complete development workflow"
        echo "  setup      Install recommended development tools"
        echo ""
        echo -e "${BLUE}Examples:${NC}"
        echo "  ./scripts/dev-tools.sh full      # Complete check"
        echo "  ./scripts/dev-tools.sh pre-commit # Before committing"
        echo "  ./scripts/dev-tools.sh setup      # One-time setup"
        ;;
esac
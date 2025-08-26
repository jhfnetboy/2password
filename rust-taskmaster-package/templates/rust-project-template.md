# Rust Project with TaskMaster Template

Complete setup template for Rust projects with TaskMaster AI and automated cache management.

## Project Setup Checklist

- [ ] Project repository created
- [ ] Rust toolchain installed
- [ ] TaskMaster installed and configured  
- [ ] PRD written and placed in `.taskmaster/docs/prd.txt`
- [ ] Tasks generated from PRD
- [ ] Complexity analysis completed
- [ ] Tasks expanded into subtasks
- [ ] Rust automation configured (rskiller + cache cleaning)
- [ ] Development environment ready

## Quick Setup Script

Save this as `setup-rust-project.sh` in your new Rust project:

```bash
#!/bin/bash
set -e

PROJECT_NAME="${1:-MyRustProject}"
PROJECT_DESC="${2:-A new Rust project with TaskMaster and automated cache management}"

echo "🦀 Setting up $PROJECT_NAME with Rust optimizations..."

# Install and setup TaskMaster
npm install -g task-master-ai
task-master init --rules claude,cursor --aliases --name="$PROJECT_NAME" --description="$PROJECT_DESC" -y
task-master models --set-main sonnet

# Install and configure Rust cache management
cargo install rskiller

# Apply Rust-specific configurations
./scripts/setup-rust-automation.sh

echo "✅ Rust project setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit .taskmaster/docs/prd.txt with your requirements"
echo "2. Run: task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0"
echo "3. Run: task-master analyze-complexity && task-master expand --all"
echo "4. Start: task-master next"
echo "5. Begin development with automated cache management!"
```

Make executable: `chmod +x setup-rust-project.sh`

Run: `./setup-rust-project.sh "My Rust Project" "My project description"`

## Rust-Specific PRD Template

Place this in `.taskmaster/docs/prd.txt`:

```markdown
# [PROJECT_NAME] - Rust Project Requirements Document

## Product Overview

### Product Positioning
[Brief description of your Rust application - CLI tool, web service, system utility, etc.]

### Target Environment
- Platform: [macOS, Linux, Windows, or cross-platform]
- Runtime: Native binary
- Dependencies: [External system requirements]

### Performance Requirements
- Binary size: [Target size limit]
- Memory usage: [Memory constraints]
- Startup time: [Performance targets]
- Throughput: [If applicable]

## Functional Requirements

### Phase 1: Core Features (MVP)

#### Feature 1: [Core Functionality]
**Description**: [What this feature does]

**Requirements**:
- [Functional requirement 1]
- [Functional requirement 2]
- [Error handling requirements]
- [Input validation requirements]

**Acceptance Criteria**:
- [ ] [Testable criterion 1]
- [ ] [Error cases handled properly]
- [ ] [Performance targets met]
- [ ] [Unit tests with >90% coverage]

**Rust-Specific Considerations**:
- Memory safety requirements
- Error handling strategy (Result<T, E>)
- Concurrency requirements (async/await, threads)
- External crate dependencies

#### Feature 2: [Additional Core Feature]
[Continue with similar structure...]

## Technical Requirements

### Technology Stack
**Core Language**: Rust (stable channel)

**Key Dependencies**:
- [tokio] - For async runtime (if needed)
- [serde] - For serialization
- [clap] - For CLI interfaces
- [anyhow/thiserror] - For error handling
- [log/tracing] - For logging

**Development Tools**:
- cargo - Build system and package manager
- rustfmt - Code formatting  
- clippy - Linting
- rust-analyzer - IDE support

### Architecture Requirements
- **Error Handling**: Use Result<T, E> pattern throughout
- **Memory Management**: Zero-copy where possible, minimal allocations
- **Concurrency**: [Specify async/sync requirements]
- **Testing**: Unit tests, integration tests, property-based testing
- **Documentation**: Comprehensive rustdoc comments

### Performance Requirements
- **Binary Size**: [Target size - use cargo bloat to measure]
- **Memory Usage**: [Target RAM usage under load]
- **CPU Usage**: [Performance benchmarks]
- **Build Time**: [Acceptable build time for CI/CD]

### Security Requirements
- **Input Validation**: All external inputs validated
- **Dependency Auditing**: cargo audit for security vulnerabilities  
- **Secrets Management**: No secrets in binary or logs
- **Safe Unsafe Code**: Minimize unsafe blocks, document necessity

### Cross-Platform Requirements (if applicable)
- **Target Platforms**: [x86_64-apple-darwin, x86_64-unknown-linux-gnu, etc.]
- **Platform-Specific Features**: [File system, network, GUI requirements]
- **CI/CD**: Cross-compilation and testing

## Development Phases

### Phase 1: Foundation (Estimated: [X] weeks)
**Timeline**: [Start date] - [End date]

**Deliverables**:
- Project structure and build system - [X days]
- Core data structures and types - [X days] 
- Basic functionality implementation - [X weeks]
- Error handling and logging - [X days]
- Unit test foundation - [X days]

**Definition of Done**:
- [ ] All core features implemented and tested
- [ ] Code passes clippy lints without warnings
- [ ] rustfmt applied to all code
- [ ] Unit test coverage >85%
- [ ] Integration tests cover main workflows
- [ ] Documentation complete with examples
- [ ] Binary builds successfully on target platforms

### Phase 2: Enhancement (Estimated: [X] weeks)
**Deliverables**:
- Performance optimizations
- Additional features
- Enhanced error reporting
- Comprehensive testing suite

### Phase 3: Production Readiness (Estimated: [X] weeks)  
**Deliverables**:
- Security audit and fixes
- Performance benchmarking
- Production deployment
- Monitoring and maintenance tools

## Testing Requirements

### Testing Strategy
**Unit Testing**:
- Coverage target: >90%
- Test all public APIs
- Test error conditions
- Property-based testing for complex logic

**Integration Testing**:
- End-to-end workflow testing
- External dependency mocking
- File system and network I/O testing

**Performance Testing**:
- Benchmarking with criterion
- Memory leak detection with valgrind
- Load testing for concurrent systems

**Security Testing**:
- Input fuzzing with cargo-fuzz
- Dependency vulnerability scanning with cargo-audit
- Static analysis with additional tools

### Quality Assurance
- Code review process for all changes
- Automated CI/CD with GitHub Actions/GitLab CI
- Pre-commit hooks with rustfmt and clippy
- Automated security scanning

## Rust-Specific Considerations

### Code Style and Standards
- Follow Rust API guidelines
- Use rustfmt with default settings
- Address all clippy warnings
- Follow Rust naming conventions

### Error Handling Strategy
- Use `anyhow` for application errors
- Use `thiserror` for library errors  
- Provide helpful error messages
- Chain errors with context

### Performance Optimization
- Profile with perf/Instruments
- Minimize allocations in hot paths
- Use appropriate data structures
- Consider zero-copy optimizations

### Dependency Management
- Minimize external dependencies
- Pin versions for reproducible builds
- Regular dependency updates
- Security audit with cargo-audit

## Success Criteria

### Launch Criteria
- [ ] All Phase 1 features complete and tested
- [ ] Performance benchmarks met
- [ ] Security audit passed
- [ ] Cross-platform builds successful
- [ ] Documentation complete
- [ ] CI/CD pipeline operational

### Post-Launch Success Metrics
- Binary size: [Target size]
- Performance: [Benchmark results]
- Memory usage: [Target memory usage]
- Build time: [CI/CD build time target]
```

## Cargo.toml Template

```toml
[package]
name = "your-project"
version = "0.1.0"
authors = ["Your Name <your.email@example.com>"]
edition = "2021"
rust-version = "1.70"
description = "A brief description of your project"
homepage = "https://github.com/yourusername/your-project"
repository = "https://github.com/yourusername/your-project"
license = "MIT OR Apache-2.0"
keywords = ["cli", "tool", "productivity"]
categories = ["command-line-utilities"]

[dependencies]
# Core dependencies
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

# Optional dependencies
log = { version = "0.4", optional = true }
tracing = { version = "0.1", optional = true }

[dev-dependencies]
criterion = "0.5"
tempfile = "3.0"
proptest = "1.0"

[features]
default = ["logging"]
logging = ["log"]
tracing = ["dep:tracing"]

[[bin]]
name = "your-project"
path = "src/main.rs"

[[bench]]
name = "benchmarks"
harness = false

[profile.dev]
# Fast builds, good for development
opt-level = 0
debug = 2
debug-assertions = true
overflow-checks = true
lto = false
panic = 'unwind'
incremental = true
codegen-units = 256

[profile.dev.package."*"]
# Optimize dependencies even in debug mode
opt-level = 3

[profile.release]
# Optimized for performance and size
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
lto = "thin"
panic = 'abort'
incremental = false
codegen-units = 1

[profile.production]
# Maximum optimization for production
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

## Development Workflow Files

### `.cargo/config.toml`
```toml
# Rust optimization configuration

[build]
# Use faster linker on macOS
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Use faster linker on Linux
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### `scripts/rust-workflow.sh`
```bash
#!/bin/bash

case "$1" in
    "dev")
        echo "🔨 Starting development session..."
        ./scripts/rust-clean.sh --analyze
        task-master next
        ;;
    "build")
        echo "🏗️  Building project..."
        cargo build --all-features
        ;;
    "test")
        echo "🧪 Running tests..."
        cargo test --all-features
        cargo clippy --all-features -- -D warnings
        ;;
    "release")
        echo "🚀 Building release..."
        cargo build --release --all-features
        ;;
    "clean")
        echo "🧹 Cleaning caches..."
        ./scripts/rust-clean.sh --safe
        ;;
    *)
        echo "Rust Development Workflow"
        echo "Usage: ./scripts/rust-workflow.sh [dev|build|test|release|clean]"
        ;;
esac
```

## Claude Code Integration

### Custom Commands

`.claude/commands/rust-build.md`:
```markdown
Build and test Rust project with comprehensive checks.

Steps:

1. Clean old artifacts: `./scripts/rust-clean.sh --safe`
2. Format code: `cargo fmt --all`
3. Run clippy: `cargo clippy --all-features -- -D warnings`
4. Run tests: `cargo test --all-features`
5. Build release: `cargo build --release --all-features`
6. Show binary size: `ls -lh target/release/[binary-name]`
```

`.claude/commands/rust-optimize.md`:
```markdown
Optimize Rust project for performance and size.

Steps:

1. Analyze binary size: `cargo bloat --release`
2. Profile performance: `cargo bench`
3. Check dependencies: `cargo tree --duplicates`
4. Security audit: `cargo audit`
5. Suggest optimizations for build time and runtime
```

## CI/CD Template

### `.github/workflows/rust.yml`
```yaml
name: Rust CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macOS-latest]
        rust: [stable, beta]

    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust toolchain
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
        override: true

    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

    - name: Cache cargo build
      uses: actions/cache@v3
      with:
        path: target
        key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

    - name: Check formatting
      run: cargo fmt --all -- --check

    - name: Run clippy
      run: cargo clippy --all-features -- -D warnings

    - name: Run tests
      run: cargo test --all-features --verbose

    - name: Build release
      run: cargo build --release --all-features
```

## Final Checklist

After setup, verify everything works:

- [ ] `cargo --version` shows Rust installed
- [ ] `rskiller --version` shows rskiller installed  
- [ ] `task-master --version` shows TaskMaster installed
- [ ] `.taskmaster/docs/prd.txt` contains your requirements
- [ ] `task-master list` shows generated tasks
- [ ] `./scripts/rust-clean.sh --analyze` shows cache status
- [ ] `cargo build` succeeds
- [ ] Claude Code responds to Rust commands

## Ready to Code!

You now have a complete Rust development environment with:
- ✅ TaskMaster AI for project management
- ✅ Automated cache cleaning every 2 hours  
- ✅ Optimized build configuration
- ✅ Custom Claude Code commands
- ✅ Comprehensive testing setup
- ✅ CI/CD templates

Happy Rust development! 🦀
#!/bin/bash

# TaskMaster Quick Setup Script for New Projects
# Usage: ./setup-taskmaster.sh [project-name] [project-description]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DEFAULT_PROJECT_NAME=$(basename "$(pwd)")
DEFAULT_DESCRIPTION="A new project with TaskMaster integration"
DEFAULT_AUTHOR="Developer"
DEFAULT_VERSION="0.1.0"

# Parse arguments
PROJECT_NAME="${1:-$DEFAULT_PROJECT_NAME}"
PROJECT_DESCRIPTION="${2:-$DEFAULT_DESCRIPTION}"
AUTHOR_NAME="${3:-$DEFAULT_AUTHOR}"
VERSION="${4:-$DEFAULT_VERSION}"

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           TaskMaster Quick Setup             ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command_exists node; then
    echo -e "${RED}❌ Node.js is not installed. Please install Node.js first.${NC}"
    exit 1
fi

if ! command_exists npm; then
    echo -e "${RED}❌ npm is not available. Please install npm first.${NC}"
    exit 1
fi

if ! command_exists git; then
    echo -e "${RED}❌ Git is not installed. Please install Git first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"
echo ""

# Check if we're in a git repository
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

# Initialize TaskMaster
echo -e "${YELLOW}Initializing TaskMaster in project...${NC}"
task-master init \
    --rules claude,cursor \
    --aliases \
    --name="$PROJECT_NAME" \
    --description="$PROJECT_DESCRIPTION" \
    --version="$VERSION" \
    --author="$AUTHOR_NAME" \
    -y

echo -e "${GREEN}✅ TaskMaster initialized${NC}"

# Configure Claude Code model
echo -e "${YELLOW}Configuring AI models...${NC}"
task-master models --set-main sonnet

echo -e "${GREEN}✅ Models configured${NC}"

# Create sample PRD if it doesn't exist
if [ ! -f ".taskmaster/docs/prd.txt" ]; then
    echo -e "${YELLOW}Creating sample PRD template...${NC}"
    cat > .taskmaster/docs/prd.txt << 'EOF'
# PROJECT_NAME - Product Requirements Document (PRD)

## Product Overview

### Product Positioning
[Describe what your product does and who it's for]

### Target Users
- [Primary user type]
- [Secondary user type]

## Product Goals

### Core Objectives
1. [Primary goal]
2. [Secondary goal]
3. [Success metrics]

## Functional Requirements

### Phase 1 (MVP) Features

#### Feature 1: [Core Feature Name]
**Description**: [What this feature does]

**Requirements**:
- [Specific requirement 1]
- [Specific requirement 2]
- [Specific requirement 3]

**Acceptance Criteria**:
- [ ] [Criterion 1]
- [ ] [Criterion 2]
- [ ] [Criterion 3]

#### Feature 2: [Another Core Feature]
**Description**: [What this feature does]

**Requirements**:
- [Requirement 1]
- [Requirement 2]

**Acceptance Criteria**:
- [ ] [Criterion 1]
- [ ] [Criterion 2]

## Technical Requirements

### Technology Stack
- [Backend technology]
- [Frontend technology]
- [Database]
- [Deployment platform]

### Performance Requirements
- [Response time requirements]
- [Scalability requirements]
- [Security requirements]

### Development Phases

#### Phase 1 (MVP) - [X weeks]
[Core functionality description]

#### Phase 2 - [X weeks]
[Extended features description]

## Testing Requirements

### Testing Strategy
- Unit testing for all core functions
- Integration testing for user flows
- Performance testing under load
- Security testing for vulnerabilities

### Acceptance Testing
- User acceptance testing scenarios
- Browser/platform compatibility testing
- Performance benchmarks

## Success Metrics
- [Metric 1: target]
- [Metric 2: target]
- [Metric 3: target]
EOF

    # Replace PROJECT_NAME placeholder
    sed -i.bak "s/PROJECT_NAME/$PROJECT_NAME/g" .taskmaster/docs/prd.txt && rm .taskmaster/docs/prd.txt.bak
    
    echo -e "${GREEN}✅ Sample PRD created at .taskmaster/docs/prd.txt${NC}"
    echo -e "${YELLOW}⚠️  Please edit .taskmaster/docs/prd.txt with your actual requirements${NC}"
fi

# Create custom Claude commands
echo -e "${YELLOW}Setting up custom Claude Code commands...${NC}"

mkdir -p .claude/commands

cat > .claude/commands/taskmaster-next.md << 'EOF'
Find the next available Task Master task and show its details.

Steps:

1. Run `task-master next` to get the next task
2. If a task is available, run `task-master show <id>` for full details
3. Provide a summary of what needs to be implemented
4. Suggest the first implementation step
EOF

cat > .claude/commands/taskmaster-complete.md << 'EOF'
Complete a Task Master task: $ARGUMENTS

Steps:

1. Review the current task with `task-master show $ARGUMENTS`
2. Verify all implementation is complete
3. Run any tests related to this task
4. Mark as complete: `task-master set-status --id=$ARGUMENTS --status=done`
5. Show the next available task with `task-master next`
EOF

cat > .claude/commands/taskmaster-status.md << 'EOF'
Show current TaskMaster project status and progress.

Steps:

1. Run `task-master list --with-subtasks` to show all tasks
2. Highlight completed vs pending tasks
3. Show next recommended task with `task-master next`
4. Provide progress summary and recommendations
EOF

echo -e "${GREEN}✅ Custom Claude Code commands created${NC}"

# Create development workflow script
cat > scripts/tm-workflow.sh << 'EOF'
#!/bin/bash

# TaskMaster Development Workflow Helper
# Usage: ./scripts/tm-workflow.sh [command]

case "$1" in
    "start")
        echo "🚀 Starting development session..."
        task-master list
        echo ""
        echo "Next task to work on:"
        task-master next
        ;;
    "next")
        echo "📋 Next available task:"
        task-master next
        ;;
    "status")
        echo "📊 Project Status:"
        task-master list --with-subtasks
        ;;
    "done")
        if [ -z "$2" ]; then
            echo "Usage: ./scripts/tm-workflow.sh done <task-id>"
            exit 1
        fi
        task-master set-status --id="$2" --status=done
        echo "✅ Task $2 marked as done!"
        echo ""
        echo "Next task:"
        task-master next
        ;;
    "progress")
        task-master set-status --id="$2" --status=in-progress
        echo "🔄 Task $2 set to in-progress"
        ;;
    *)
        echo "TaskMaster Workflow Helper"
        echo "Usage: ./scripts/tm-workflow.sh [command]"
        echo ""
        echo "Commands:"
        echo "  start     - Show project status and next task"
        echo "  next      - Show next available task"
        echo "  status    - Show all tasks with subtasks"
        echo "  done <id> - Mark task as done and show next"
        echo "  progress <id> - Mark task as in-progress"
        ;;
esac
EOF

chmod +x scripts/tm-workflow.sh

echo -e "${GREEN}✅ Development workflow script created${NC}"

# Final instructions
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║            Setup Complete! 🎉                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Next Steps:${NC}"
echo ""
echo -e "${YELLOW}1. Edit your PRD:${NC}"
echo "   Edit .taskmaster/docs/prd.txt with your actual requirements"
echo ""
echo -e "${YELLOW}2. Generate tasks:${NC}"
echo "   task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0"
echo ""
echo -e "${YELLOW}3. Analyze complexity:${NC}"
echo "   task-master analyze-complexity"
echo ""
echo -e "${YELLOW}4. Expand tasks:${NC}"
echo "   task-master expand --all"
echo ""
echo -e "${YELLOW}5. Start development:${NC}"
echo "   ./scripts/tm-workflow.sh start"
echo ""
echo -e "${YELLOW}Daily workflow:${NC}"
echo "   ./scripts/tm-workflow.sh next      # See next task"
echo "   ./scripts/tm-workflow.sh done 1.1  # Mark task done"
echo "   ./scripts/tm-workflow.sh status    # Project overview"
echo ""
echo -e "${GREEN}Happy coding with TaskMaster! 🚀${NC}"
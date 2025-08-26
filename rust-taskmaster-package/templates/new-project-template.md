# New Project with TaskMaster Template

Copy and customize this template for your new project.

## Project Setup Checklist

- [ ] Project repository created
- [ ] TaskMaster installed and configured  
- [ ] PRD written and placed in `.taskmaster/docs/prd.txt`
- [ ] Tasks generated from PRD
- [ ] Complexity analysis completed
- [ ] Tasks expanded into subtasks
- [ ] Development environment ready

## Quick Setup Script

Save this as `setup-project.sh` in your new project:

```bash
#!/bin/bash
set -e

PROJECT_NAME="${1:-MyProject}"
PROJECT_DESC="${2:-A new project with TaskMaster}"

echo "🚀 Setting up $PROJECT_NAME with TaskMaster..."

# Install and setup TaskMaster
npm install -g task-master-ai
task-master init --rules claude,cursor --aliases --name="$PROJECT_NAME" --description="$PROJECT_DESC" -y
task-master models --set-main sonnet

echo "✅ TaskMaster setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit .taskmaster/docs/prd.txt with your requirements"
echo "2. Run: task-master parse-prd .taskmaster/docs/prd.txt --num-tasks=0"
echo "3. Run: task-master analyze-complexity && task-master expand --all"
echo "4. Start: task-master next"
```

Make executable: `chmod +x setup-project.sh`

Run: `./setup-project.sh "My Project Name" "My project description"`

## PRD Template

Place this in `.taskmaster/docs/prd.txt`:

```markdown
# [PROJECT_NAME] - Product Requirements Document

## Product Overview

### Product Positioning
[Brief description of what your product does and who it's for]

### Target Users
- [Primary user group]
- [Secondary user group]
- [Use cases and scenarios]

### Success Metrics
- [Key metric 1 with target]
- [Key metric 2 with target]
- [Key metric 3 with target]

## Product Goals

### Primary Objectives
1. [Main goal - what problem are you solving?]
2. [Secondary goal]
3. [Tertiary goal]

### User Stories
- As a [user type], I want [functionality] so that [benefit]
- As a [user type], I want [functionality] so that [benefit]
- As a [user type], I want [functionality] so that [benefit]

## Functional Requirements

### Phase 1: Core Features (MVP)

#### Feature 1: [Core Feature Name]
**Description**: [Detailed description of what this feature does]

**User Stories**:
- As a user, I want to [action] so that [benefit]
- As a user, I want to [action] so that [benefit]

**Requirements**:
- [Specific functional requirement 1]
- [Specific functional requirement 2]  
- [Specific functional requirement 3]
- [Integration requirement if applicable]
- [Data requirement if applicable]

**Acceptance Criteria**:
- [ ] [Testable criterion 1]
- [ ] [Testable criterion 2]
- [ ] [Testable criterion 3]
- [ ] [Performance criterion if applicable]
- [ ] [Security criterion if applicable]

**Priority**: High/Medium/Low

#### Feature 2: [Another Core Feature]
**Description**: [What this feature does and why it's important]

**User Stories**:
- [User story 1]
- [User story 2]

**Requirements**:
- [Requirement 1]
- [Requirement 2]
- [Requirement 3]

**Acceptance Criteria**:
- [ ] [Criterion 1]
- [ ] [Criterion 2]
- [ ] [Criterion 3]

**Priority**: High/Medium/Low

#### Feature 3: [Additional Feature]
[Continue with more features following the same pattern...]

### Phase 1 Integration Requirements
- [External service integration 1]
- [External service integration 2]
- [Database requirements]
- [Authentication/authorization needs]

## Technical Requirements

### Technology Stack
**Backend**:
- [Primary language/framework]
- [Database technology]
- [Server/hosting platform]

**Frontend** (if applicable):
- [Framework/library]
- [UI component library]
- [State management]

**DevOps**:
- [CI/CD platform]
- [Deployment strategy]
- [Monitoring tools]

### Architecture Requirements
- [Architectural pattern - microservices/monolith/etc]
- [Scalability requirements]
- [Performance requirements]
- [Data storage and retrieval patterns]

### Security Requirements
- [Authentication method]
- [Authorization model]  
- [Data encryption requirements]
- [Privacy compliance needs]
- [Security testing requirements]

### Performance Requirements
- [Response time requirements]
- [Throughput requirements]
- [Concurrent user capacity]
- [Resource usage limits]

### Compatibility Requirements
- [Browser support]
- [Mobile device support]
- [Operating system support]
- [Third-party integration compatibility]

## User Experience Requirements

### User Interface
- [Design principles]
- [Accessibility requirements]
- [Responsive design needs]
- [User workflow descriptions]

### User Onboarding
- [New user flow]
- [Learning curve expectations]
- [Help/documentation needs]

## Testing Requirements

### Testing Strategy
**Unit Testing**:
- [Coverage requirements]
- [Testing frameworks]
- [Key areas to test]

**Integration Testing**:
- [API testing approach]
- [Database integration testing]
- [Third-party service testing]

**End-to-End Testing**:
- [User journey testing]
- [Critical path testing]
- [Cross-browser testing]

**Performance Testing**:
- [Load testing requirements]
- [Stress testing scenarios]
- [Performance benchmarks]

**Security Testing**:
- [Vulnerability assessment]
- [Penetration testing]
- [Security audit requirements]

### Quality Assurance
- [Code review process]
- [Testing sign-off criteria]
- [Bug triage process]

## Development Phases

### Phase 1: MVP (Estimated: [X] weeks)
**Timeline**: [Start date] - [End date]

**Deliverables**:
- [Core feature 1] - [X weeks]
- [Core feature 2] - [X weeks]
- [Core feature 3] - [X weeks]
- [Basic testing and documentation] - [X weeks]

**Definition of Done**:
- [ ] All core features implemented and tested
- [ ] Basic documentation completed
- [ ] Performance requirements met
- [ ] Security requirements satisfied
- [ ] User acceptance testing passed

### Phase 2: Enhancement (Estimated: [X] weeks)
**Timeline**: [Start date] - [End date]

**Deliverables**:
- [Enhanced feature 1]
- [Additional feature 1]
- [Performance optimizations]
- [Advanced testing]

### Phase 3: Scale and Polish (Estimated: [X] weeks)
**Timeline**: [Start date] - [End date]

**Deliverables**:
- [Scalability improvements]
- [Advanced features]
- [Production deployment]
- [Monitoring and maintenance tools]

## Risk Assessment

### Technical Risks
- [Risk 1]: [Impact] - [Mitigation strategy]
- [Risk 2]: [Impact] - [Mitigation strategy]
- [Risk 3]: [Impact] - [Mitigation strategy]

### Project Risks
- [Resource risk]: [Mitigation]
- [Timeline risk]: [Mitigation]
- [Scope risk]: [Mitigation]

### Dependencies
- [External dependency 1]: [Risk level] - [Backup plan]
- [External dependency 2]: [Risk level] - [Backup plan]

## Success Criteria

### Launch Criteria
- [ ] All Phase 1 features complete and tested
- [ ] Performance benchmarks met
- [ ] Security audit passed
- [ ] User acceptance testing completed
- [ ] Documentation finalized
- [ ] Deployment pipeline ready

### Post-Launch Success Metrics
- [Metric 1]: [Target value] within [timeframe]
- [Metric 2]: [Target value] within [timeframe]
- [Metric 3]: [Target value] within [timeframe]

## Appendix

### Assumptions
- [Assumption 1]
- [Assumption 2]
- [Assumption 3]

### Constraints
- [Technical constraint 1]
- [Resource constraint 1]
- [Timeline constraint 1]

### Open Questions
- [Question 1 requiring resolution]
- [Question 2 requiring resolution]
- [Question 3 requiring resolution]
```

## Development Workflow Files

### `.claude/commands/taskmaster-workflow.md`
```markdown
TaskMaster development workflow helper: $ARGUMENTS

Available workflows:
- "start" - Begin development session with task overview
- "next" - Show next available task  
- "complete X.Y" - Mark task complete and show next
- "status" - Show project progress overview
- "add [description]" - Add new task
- "expand X" - Break down task into subtasks

Steps:
1. Run the requested workflow command
2. Show relevant task information  
3. Provide next step recommendations
4. Update task status if needed
```

### `scripts/dev-workflow.sh`
```bash
#!/bin/bash

case "$1" in
    "start")
        echo "🚀 Development Session Starting..."
        task-master list --status=in-progress
        echo ""
        echo "📋 Next available task:"
        task-master next
        ;;
    "next")
        task-master next
        ;;
    "status")
        echo "📊 Project Progress:"
        task-master list
        ;;
    "done")
        if [ -z "$2" ]; then
            echo "Usage: ./scripts/dev-workflow.sh done <task-id>"
            exit 1
        fi
        task-master set-status --id="$2" --status=done
        echo "✅ Task $2 completed!"
        task-master next
        ;;
    *)
        echo "TaskMaster Development Workflow"
        echo "Usage: ./scripts/dev-workflow.sh [start|next|status|done <id>]"
        ;;
esac
```

Make executable: `chmod +x scripts/dev-workflow.sh`

## Final Checklist

After setup, verify everything works:

- [ ] `task-master --version` shows version
- [ ] `task-master models` shows sonnet configured
- [ ] `.taskmaster/docs/prd.txt` contains your requirements
- [ ] `task-master list` shows generated tasks
- [ ] Claude Code responds to "Show me my current tasks"
- [ ] `./scripts/dev-workflow.sh start` works

## Ready to Code!

You now have a complete TaskMaster setup that can be replicated for any project. The AI will help you stay organized and focused throughout development.
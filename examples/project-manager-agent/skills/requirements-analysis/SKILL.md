---
name: requirements-analysis
description: Collect, prioritize, and validate requirements into a structured Product Requirements Document
version: "1.0.0"
author: developer
triggers:
  - analyze requirements
  - 需求分析
  - PRD
  - product requirements
  - gather requirements
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Requirements Analysis Skill

## Execution Steps

1. **Collect requirements**
   - Use `memory_recall` to retrieve any existing project context, stakeholder preferences, and prior requirements
   - Use `file_read` to examine any existing documentation, user stories, or meeting notes
   - Identify all requirement sources: stakeholders, market research, technical constraints, regulatory needs
   - Distinguish between stated requirements (what people say they want) and latent requirements (what they actually need)
   - Document each requirement with a unique ID (REQ-001, REQ-002, etc.)

2. **Prioritize requirements**
   - Apply MoSCoW prioritization to each requirement:
     - **Must Have**: Non-negotiable for launch (without these, the product fails)
     - **Should Have**: Important but not critical (workarounds exist)
     - **Could Have**: Nice-to-have if time and resources allow
     - **Won't Have**: Explicitly out of scope for this release
   - Validate priorities with stakeholders — ensure alignment between business and technical perspectives
   - Identify dependencies between requirements (some Must-Haves may depend on others)

3. **Assess feasibility**
   - For each requirement, evaluate:
     - **Technical feasibility**: Can it be built with current technology and team skills?
     - **Schedule feasibility**: Can it be delivered within the timeline?
     - **Cost feasibility**: Is the investment justified by the expected return?
   - Flag high-risk requirements that need proof-of-concept or spike investigation
   - Identify requirements that conflict with each other

4. **Write the PRD**
   - Use `file_write` to create a structured Product Requirements Document
   - Include all sections per the output format below
   - Each functional requirement should have clear acceptance criteria
   - Use `memory_store` to persist the PRD location and key decisions for future reference

5. **Review and confirm**
   - Present the PRD for stakeholder review
   - Track open questions and unresolved items
   - Document sign-off: who approved, when, and any conditions

## Output Format

```markdown
# PRD: [Product/Feature Name]

## 1. Overview
### 1.1 Problem Statement
[What problem are we solving? For whom?]

### 1.2 Goals & Non-Goals
**Goals:**
- [What this project will achieve]

**Non-Goals:**
- [What this project explicitly will NOT address]

## 2. Requirements
### Functional Requirements
| ID | Requirement | Priority | Acceptance Criteria | Dependency |
|----|------------|----------|---------------------|------------|
| REQ-001 | [Description] | Must | [Testable criteria] | — |
| REQ-002 | [Description] | Should | [Testable criteria] | REQ-001 |

### Non-Functional Requirements
| ID | Requirement | Metric | Target |
|----|------------|--------|--------|
| NFR-001 | Performance | Page load time | < 2s |

## 3. User Stories
- As a [role], I want to [action], so that [benefit]

## 4. Assumptions & Constraints
- [List of assumptions made and known constraints]

## 5. Open Questions
- [ ] [Unresolved question requiring stakeholder input]

## 6. Approval
| Role | Name | Date | Status |
|------|------|------|--------|
| PM | | | Pending |
| Tech Lead | | | Pending |
```

## Notes

- Requirements should be testable — if you can't write an acceptance criterion, the requirement is too vague
- Separate "what" from "how" — the PRD describes what the product does, not how it's implemented
- Always document non-goals explicitly to prevent scope creep
- Use `memory_store` to track requirement changes and their rationale over time

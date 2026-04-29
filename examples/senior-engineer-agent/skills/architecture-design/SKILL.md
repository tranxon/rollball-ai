---
name: architecture-design
description: Design system architecture with requirement analysis, technology selection, module decomposition, and interface definition
version: "1.0.0"
author: developer
triggers:
  - design
  - architecture
  - 系统设计
  - system design
  - 架构设计
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Architecture Design Skill

## Execution Steps

1. **Requirement analysis**
   - Use `memory_recall` to retrieve any prior context about the project, constraints, and decisions
   - Use `file_read` to examine existing codebase structure if the project already exists
   - Identify functional requirements: what must the system do?
   - Identify non-functional requirements: performance, scalability, security, reliability
   - Identify constraints: technology stack, team skills, timeline, budget
   - List explicit requirements and inferred requirements separately

2. **Technology selection**
   - Evaluate candidate technologies against requirements
   - Consider team familiarity and ecosystem maturity
   - Assess trade-offs: e.g., Rust for safety vs. Python for speed of development
   - Document the selection rationale — why this technology over alternatives?
   - Identify integration points between selected technologies

3. **Module decomposition**
   - Identify bounded contexts and domain boundaries
   - Define module responsibilities using single-responsibility principle
   - Map data flow between modules
   - Identify shared infrastructure concerns (logging, error handling, configuration)
   - Document module dependency graph (which module depends on which)

4. **Interface design**
   - Define public APIs for each module (function signatures, data types, protocols)
   - Specify communication patterns: synchronous vs. asynchronous, request-response vs. event-driven
   - Define error contracts: what errors each interface can return and what they mean
   - Design configuration interfaces: how modules are configured and initialized
   - Sketch the data models and their relationships

5. **Output the design document**
   - Use `file_write` to save the architecture document
   - Use `memory_store` to persist key architectural decisions for future reference

## Output Format

```markdown
# Architecture Design: [System Name]

## 1. Overview
[One-paragraph description of the system and its purpose]

## 2. Requirements
### Functional Requirements
- FR-1: [description]
### Non-Functional Requirements
- NFR-1: [description]
### Constraints
- C-1: [description]

## 3. Technology Stack
| Layer | Technology | Rationale |
|-------|-----------|-----------|
| ...   | ...       | ...       |

## 4. Module Architecture
[Module diagram or description]

### Module: [Name]
- **Responsibility**: [what it does]
- **Dependencies**: [what it depends on]
- **Public Interface**: [key APIs]

## 5. Data Flow
[Description of how data moves through the system]

## 6. Key Decisions
| Decision | Choice | Rationale | Trade-offs |
|----------|--------|-----------|------------|
| ...      | ...    | ...       | ...        |

## 7. Open Questions
- [Unresolved items requiring further discussion]
```

## Notes

- Architecture is about trade-offs — always document what you chose AND what you chose not to do
- Prefer simple architectures over elegant ones; complexity should be earned, not assumed
- When existing code is present, design incrementally — minimize disruption to working systems
- Store architectural decisions in `memory_store` so they persist across sessions
- If requirements are ambiguous, list assumptions explicitly and flag them for validation

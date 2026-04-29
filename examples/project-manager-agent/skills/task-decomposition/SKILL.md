---
name: task-decomposition
description: Break down project goals into actionable tasks with effort estimates and dependency mapping
version: "1.0.0"
author: developer
triggers:
  - break down
  - 任务分解
  - WBS
  - work breakdown
  - decompose
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Task Decomposition Skill

## Execution Steps

1. **Understand the goal**
   - Use `memory_recall` to retrieve the project PRD, prior task breakdowns, and team velocity data
   - Use `file_read` to examine any existing plans or requirements documents
   - Clearly state the project goal or milestone being decomposed
   - Identify the scope boundary: what is in scope and what is not
   - Determine the level of granularity needed (epic-level, feature-level, or task-level)

2. **Decompose into sub-tasks**
   - Apply the MECE principle (Mutually Exclusive, Collectively Exhaustive)
   - Start with major deliverables, then decompose each into smaller work items
   - Each task should be completable by one person in 1-3 days (if larger, decompose further)
   - Use action verbs: "Implement", "Design", "Test", "Deploy", "Document"
   - Include non-coding tasks: documentation, testing, DevOps setup, stakeholder reviews

3. **Estimate effort**
   - Use three-point estimation for uncertain tasks: Optimistic, Most Likely, Pessimistic
   - Expected = (O + 4M + P) / 6 (PERT formula)
   - Reference historical data from `memory_recall` for similar past tasks
   - Account for non-productive time: meetings, code reviews, context switching (apply 0.7-0.8 utilization factor)
   - Include buffer for unknown unknowns (10-20% depending on uncertainty)

4. **Determine dependencies**
   - Map task dependencies: Finish-to-Start (FS), Start-to-Start (SS), Finish-to-Finish (FF)
   - Identify the critical path: the longest chain of dependent tasks
   - Flag tasks that can run in parallel (no dependencies between them)
   - Identify external dependencies: third-party APIs, cross-team deliverables, approval gates
   - Document any assumptions about dependency resolution

5. **Output the task list**
   - Use `file_write` to save the task breakdown
   - Use `memory_store` to persist the task structure for progress tracking

## Output Format

```markdown
# Task Breakdown: [Project/Milestone Name]

## Overview
- **Goal**: [What we're delivering]
- **Scope**: [What's included]
- **Total estimated effort**: [person-days/weeks]

## Task List

| ID | Task | Owner | Estimate | Priority | Dependencies | Status |
|----|------|-------|----------|----------|-------------|--------|
| T-001 | [Task description] | [Role] | 2d | Must | — | Not Started |
| T-002 | [Task description] | [Role] | 1d | Must | T-001 | Not Started |
| T-003 | [Task description] | [Role] | 3d | Should | T-001 | Not Started |

## Dependency Graph
```
T-001 ──→ T-002 ──→ T-005
   └──────→ T-003 ──→ T-005
   └──────→ T-004 (parallel)
```

## Critical Path
T-001 → T-002 → T-005 ([X] days)

## Risk Notes
- T-003 depends on external API; spike needed to validate integration
- T-002 estimate is uncertain; may need +2 days if edge cases are complex
```

## Notes

- Tasks should be small enough to track progress daily but large enough to be meaningful
- Include explicit "Definition of Done" for each task if it's non-obvious
- When in doubt about estimates, round up — underestimation is far more common than overestimation
- Use `memory_store` to save estimation data so future estimates can reference historical accuracy
- Revisit the breakdown as the project progresses — initial decomposition is never perfect

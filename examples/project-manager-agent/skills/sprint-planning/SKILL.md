---
name: sprint-planning
description: Plan sprint scope based on backlog priorities, team capacity, and historical velocity
version: "1.0.0"
author: developer
triggers:
  - sprint
  - 迭代规划
  - sprint planning
  - plan sprint
  - iteration planning
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Sprint Planning Skill

## Execution Steps

1. **Review the previous sprint**
   - Use `memory_recall` to retrieve the previous sprint's results, velocity, and retrospective action items
   - Assess what was completed vs. planned (completion rate)
   - Carry over unfinished items to the new sprint backlog
   - Review retrospective improvements to implement in this sprint
   - Note any changes in team composition or availability

2. **Evaluate the backlog**
   - Use `file_read` to examine the product backlog and any newly added items
   - Ensure top-priority items have clear acceptance criteria
   - Identify items that are "sprint-ready" (well-defined, estimated, no open questions)
   - Flag items that need further refinement before they can be committed
   - Consider the product owner's priorities and any deadlines driving the sprint

3. **Calculate capacity**
   - Determine available person-days for the sprint:
     - Total sprint days × number of team members
     - Subtract: holidays, time off, meetings, on-call duties
     - Apply utilization factor (typically 0.7-0.8) for non-task overhead
   - Reference the team's average velocity from `memory_recall` (last 3 sprints)
   - Account for carry-over items from the previous sprint
   - Adjust for known disruptions: onboarding, production incidents, etc.

4. **Select sprint items**
   - Start with the highest-priority backlog items
   - Ensure each selected item has a clear definition of done
   - Leave a 10-15% buffer for unplanned work (bugs, support, emergencies)
   - Verify that selected items align with the sprint goal
   - Confirm that no individual is over-committed (> 100% of their capacity)
   - Ensure a mix of feature work, tech debt, and bug fixes

5. **Define the sprint goal**
   - Articulate a clear, measurable sprint goal that ties the selected items together
   - The goal should be understandable by non-technical stakeholders
   - All committed items should contribute to the sprint goal
   - Use `file_write` to save the sprint plan
   - Use `memory_store` to persist the sprint commitment for tracking

## Output Format

```markdown
# Sprint Plan: Sprint [N]

## Sprint Goal
[One clear sentence describing what the sprint aims to deliver]

## Sprint Parameters
- **Duration**: [X] weeks ([start date] — [end date])
- **Team Capacity**: [Y] story points / [Z] person-days
- **Average Velocity**: [V] story points (last 3 sprints)

## Committed Items
| ID | Item | Story Points | Owner | Priority | Notes |
|----|------|-------------|-------|----------|-------|
| PB-001 | [Description] | 5 | [Name] | Must | [Context] |
| PB-002 | [Description] | 3 | [Name] | Must | [Context] |
| PB-003 | [Description] | 8 | [Name] | Should | [Context] |

**Total Committed**: [X] story points

## Carry-Over from Sprint [N-1]
| ID | Item | Remaining | Reason |
|----|------|-----------|--------|
| PB-0XX | [Description] | 2 pts | [Why not completed] |

## Buffer Allocation
- Unplanned work buffer: [X] points ([Y]% of capacity)
- On-call / support rotation: [Z] person-days

## Risks & Dependencies
| Item | Risk/Dependency | Mitigation |
|------|----------------|------------|
| PB-001 | [Dependency description] | [Contingency] |

## Retrospective Actions from Sprint [N-1]
- [ ] [Action from last retrospective]
- [ ] [Action from last retrospective]
```

## Notes

- Under-commit and over-deliver is better than over-commit and under-deliver
- The sprint goal is a commitment, the individual stories are a forecast — be clear about this distinction
- If the team is consistently not meeting sprint commitments, reduce velocity estimates
- Use `memory_store` to track velocity trends and sprint completion rates over time
- Sprint planning should take no more than 2 hours for a 2-week sprint — time-box it

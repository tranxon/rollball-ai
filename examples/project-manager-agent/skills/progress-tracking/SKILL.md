---
name: progress-tracking
description: Track project progress against plan, identify deviations, and produce status reports
version: "1.0.0"
author: developer
triggers:
  - track progress
  - 进度跟踪
  - status update
  - weekly report
  - project status
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Progress Tracking Skill

## Execution Steps

1. **Collect progress from all sources**
   - Use `memory_recall` to retrieve the project plan, task list, and previous status reports
   - Use `file_read` to examine any project tracking files, changelogs, or commit logs
   - Gather status from each team member or work stream:
     - What was completed since the last check-in?
     - What is currently in progress?
     - What is blocked or at risk?
   - Collect quantitative metrics: tasks completed, story points delivered, bugs resolved

2. **Compare against plan**
   - Compare actual progress to the planned schedule
   - Calculate schedule variance: (Earned Value - Planned Value) / Planned Value
   - Identify which tasks are on track, ahead, or behind
   - Assess burndown/burnup status for the current sprint
   - Track velocity trend: is the team delivering at the expected rate?

3. **Identify deviations**
   - **Schedule deviation**: Tasks that are behind their planned completion date
   - **Scope deviation**: New requirements or changes not in the original plan
   - **Quality deviation**: Increase in bug rates or test failures
   - **Resource deviation**: Team member availability changes, tooling issues
   - For each deviation, determine the root cause and whether it's a one-time event or a trend

4. **Formulate adjustment plan**
   - For minor deviations (< 10%): adjust within the sprint, no escalation needed
   - For moderate deviations (10-20%): re-prioritize backlog, adjust scope or timeline
   - For major deviations (> 20%): escalate to stakeholders with options and trade-offs
   - Always present multiple options: "We can do A (trade-off: X), B (trade-off: Y), or C (trade-off: Z)"
   - Document the chosen adjustment and its expected impact

5. **Output the status report**
   - Use `file_write` to save the status report
   - Use `memory_store` to persist the status data and trend information

## Output Format

```markdown
# Project Status Report — [Date]

## Executive Summary
[2-3 sentence overview: on track / at risk / off track, and the #1 thing to know]

## Progress Overview
| Metric | Plan | Actual | Variance |
|--------|------|--------|----------|
| Sprint completion | 80% | 65% | -15% |
| Story points delivered | 21 | 17 | -4 |
| Open bugs | 5 | 8 | +3 |

## Completed This Period
- [x] [Task/deliverable completed]
- [x] [Task/deliverable completed]

## In Progress
- [ ] [Task in progress] — [Owner] — [Expected completion]
- [ ] [Task in progress] — [Owner] — [Expected completion]

## Blocked / At Risk
- [ ] [Task] — Blocker: [description] — Action needed: [what needs to happen]

## Adjustments Made
- [Description of any scope, schedule, or resource changes]

## Key Risks & Actions
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| [Risk] | High/Med/Low | High/Med/Low | [Action] |

## Next Period Focus
1. [Top priority for the next period]
2. [Second priority]
3. [Third priority]
```

## Notes

- Status reports should be factual and concise — no burying bad news
- Always include the "so what" — don't just report numbers, explain their implications
- Red/yellow/green status indicators should be used consistently: define what each means for your project
- Use `memory_store` to track velocity trends and estimation accuracy over time
- When the project is off track, present the recovery plan alongside the bad news

---
name: meeting-notes
description: Capture, organize, and distribute structured meeting notes with decisions and action items
version: "1.0.0"
author: developer
triggers:
  - meeting
  - 会议纪要
  - meeting notes
  - meeting minutes
  - take notes
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Meeting Notes Skill

## Execution Steps

1. **Record the agenda**
   - Use `memory_recall` to retrieve context about ongoing projects and previous action items
   - Document the meeting purpose and expected outcomes
   - List the agenda items with allocated time for each
   - Record all attendees and their roles
   - Note any invitees who could not attend (absent stakeholders)

2. **Summarize discussion points**
   - For each agenda item, capture the key discussion points
   - Distinguish between facts presented, opinions expressed, and questions raised
   - Note areas of agreement and disagreement
   - Capture the reasoning behind significant points (not just conclusions)
   - If technical details are discussed, record enough context for non-attendees to understand

3. **Document decisions**
   - For each decision made, record:
     - **What was decided**: The specific decision and its scope
     - **Why**: The rationale and key factors considered
     - **Who decided**: Who was involved in making the decision
     - **Conditions**: Any conditions, caveats, or review dates attached
   - Flag decisions that are provisional (need confirmation from absent stakeholders)
   - Note decisions that were explicitly deferred to a future meeting

4. **Assign action items**
   - For each action item, specify:
     - **What**: Clear description of the task
     - **Who**: Single responsible owner (not "the team")
     - **When**: Specific due date (not "soon" or "ASAP")
     - **Context**: Why this action is needed and what it depends on
   - Number action items for easy tracking (AI-001, AI-002, etc.)
   - Check that every action item has an owner present in the meeting
   - Use `memory_store` to persist action items for follow-up tracking

5. **Distribute the notes**
   - Use `file_write` to save the formatted meeting notes
   - Distribute within 24 hours of the meeting (sooner is better)
   - Highlight decisions and action items at the top for quick scanning
   - Include a link to any related documents or follow-up meetings

## Output Format

```markdown
# Meeting Notes: [Meeting Title]

**Date**: [YYYY-MM-DD]
**Time**: [Start] — [End]
**Attendees**: [List of attendees with roles]
**Absent**: [List of absent invitees]

## Agenda
1. [Topic 1] — [Time allocation]
2. [Topic 2] — [Time allocation]
3. [Topic 3] — [Time allocation]

## Decisions
| # | Decision | Rationale | Decided By | Conditions |
|---|----------|-----------|------------|------------|
| D-001 | [Decision] | [Why] | [Who] | [Any caveats] |
| D-002 | [Decision] | [Why] | [Who] | [Any caveats] |

## Discussion Summary
### [Agenda Item 1]
- [Key point 1]
- [Key point 2]
- [Dissenting opinion or concern raised]

### [Agenda Item 2]
- [Key point 1]
- [Key point 2]

## Action Items
| # | Action | Owner | Due Date | Status |
|---|--------|-------|----------|--------|
| AI-001 | [Task description] | [Name] | [Date] | Open |
| AI-002 | [Task description] | [Name] | [Date] | Open |

## Next Meeting
- **Date**: [Next meeting date]
- **Carry-over items**: [Items deferred from this meeting]

## Appendix
- [Links to documents, diagrams, or references mentioned]
```

## Notes

- Write notes during the meeting, not after — memory degrades rapidly
- Focus on decisions and action items — these are the most valuable outputs
- If you're unsure about a decision or action item, confirm it before the meeting ends
- Keep discussion summaries concise — the goal is understanding, not transcription
- Use `memory_store` to track action item completion rates across meetings

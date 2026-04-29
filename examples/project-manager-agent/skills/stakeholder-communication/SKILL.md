---
name: stakeholder-communication
description: Prepare and deliver targeted communications to project stakeholders with appropriate detail and format
version: "1.0.0"
author: developer
triggers:
  - communicate
  - 汇报
  - stakeholder update
  - executive summary
  - stakeholder communication
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Stakeholder Communication Skill

## Execution Steps

1. **Identify the audience**
   - Use `memory_recall` to retrieve stakeholder profiles, communication preferences, and prior feedback
   - Determine who needs this communication:
     - **Executives**: Need high-level status, business impact, and decisions needed
     - **Product owners**: Need feature progress, scope changes, and timeline impacts
     - **Engineering leads**: Need technical details, blockers, and resource needs
     - **Team members**: Need task assignments, dependencies, and context
     - **External partners**: Need deliverable status and integration timelines
   - Identify the decision-maker: who can act on this information?

2. **Prepare key messages**
   - Define the 3 key messages you need to convey (people remember 3 things max)
   - Structure messages using the Minto Pyramid:
     - **Top**: The answer or recommendation first
     - **Middle**: Supporting arguments and evidence
     - **Bottom**: Detailed data and analysis
   - For each message, ask "So what?" — ensure there's a clear implication or action
   - Anticipate questions and prepare concise answers
   - Quantify whenever possible: "3 days behind" not "slightly delayed"

3. **Choose the communication format**
   - Match the format to the audience and urgency:
     - **Executive email**: 3 paragraphs max, lead with status and decisions needed
     - **Slack/Teams update**: Bullet points, highlight blockers, link to details
     - **Presentation slides**: One idea per slide, visual data, appendix for details
     - **1-on-1 briefing**: Conversation guide with key points and discussion prompts
     - **Written report**: Structured document with executive summary and sections
   - Adapt the level of technical detail to the audience

4. **Execute the communication**
   - Use `file_write` to create the communication artifact
   - Lead with the most important information (inverted pyramid style)
   - Be transparent about bad news — frame it as "here's the situation and here's our plan"
   - Clearly state any decisions or actions needed from the audience
   - Include a specific deadline for any requested response or action

5. **Record and follow up**
   - Use `memory_store` to persist:
     - What was communicated, to whom, and when
     - Decisions or commitments made by stakeholders
     - Feedback received (positive or negative)
     - Action items arising from the communication
   - Set a follow-up date for any pending decisions or actions
   - Track whether stakeholders are responsive and adjust approach if needed

## Output Format

### Executive Update Template
```markdown
# [Project Name] — Executive Update

## Status: [On Track / At Risk / Off Track]

## Key Messages
1. [Most important message]
2. [Second message]
3. [Third message]

## Decisions Needed
| Decision | Options | Recommendation | Deadline |
|----------|---------|---------------|----------|
| [What needs to be decided] | [A/B/C] | [Recommended option] | [Date] |

## Timeline Impact
[None / [X] days delay / Scope reduction needed]

## Budget Impact
[None / [$X] over budget / [X]% utilization]
```

### Team Update Template
```markdown
# [Project Name] — Team Update

## This Week
- [x] [Completed item]
- [x] [Completed item]

## Next Week Focus
1. [Priority 1]
2. [Priority 2]

## Blockers
- [Blocker and who can help resolve it]

## Shout-outs
- [Recognition for good work]
```

## Notes

- The most important part of communication is knowing your audience — one size does not fit all
- Bad news should travel fast — don't sit on problems hoping they'll resolve themselves
- When presenting options, always include your recommendation with rationale
- Follow up written communications with verbal confirmation for critical items
- Use `memory_store` to build stakeholder profiles over time: preferred format, response patterns, decision style

---
name: risk-assessment
description: Identify, evaluate, and plan mitigation for project risks using structured risk analysis
version: "1.0.0"
author: developer
triggers:
  - risk
  - 风险评估
  - assess risk
  - risk analysis
  - risk register
tool_deps:
  - file_read
  - file_write
  - memory_store
  - memory_recall
---

# Risk Assessment Skill

## Execution Steps

1. **Identify risks**
   - Use `memory_recall` to retrieve historical risk data from past projects and known issues
   - Use `file_read` to examine project plans, technical architecture, and dependency lists
   - Brainstorm risks across categories:
     - **Technical**: Technology uncertainty, integration complexity, performance risks, security vulnerabilities
     - **Schedule**: Optimistic estimates, dependency delays, scope creep
     - **Resource**: Key person dependency, skill gaps, team turnover
     - **External**: Third-party changes, regulatory shifts, market conditions
   - Involve team members — risks are often known to those closest to the work
   - Document each risk with a unique ID (RSK-001, RSK-002, etc.)

2. **Evaluate impact × probability**
   - For each identified risk, rate **Impact** and **Probability** on a 1-5 scale:
     - **Impact**: 1=Negligible, 2=Minor, 3=Moderate, 4=Major, 5=Critical
     - **Probability**: 1=Very Unlikely, 2=Unlikely, 3=Possible, 4=Likely, 5=Very Likely
   - Calculate **Risk Score** = Impact × Probability (range 1-25)
   - Classify risks:
     - **High** (15-25): Requires active mitigation and monitoring
     - **Medium** (6-14): Requires mitigation plan, monitor periodically
     - **Low** (1-5): Accept and monitor; no active mitigation needed

3. **Develop mitigation strategies**
   - For each High and Medium risk:
     - **Avoid**: Change the plan to eliminate the risk
     - **Transfer**: Shift the risk to another party (insurance, outsourcing, contract)
     - **Mitigate**: Reduce probability (preventive action) or reduce impact (contingency plan)
     - **Accept**: Acknowledge the risk and set a trigger for action if it materializes
   - Define specific, actionable mitigation steps with owners and deadlines
   - For each mitigation, estimate the residual risk (risk after mitigation is applied)

4. **Create monitoring plan**
   - Define risk triggers: observable indicators that a risk is materializing
   - Assign risk owners responsible for monitoring each risk
   - Set review cadence: High risks weekly, Medium risks bi-weekly, Low risks monthly
   - Define escalation criteria: when does a risk owner need to involve others?
   - Schedule risk review as a standing agenda item in team meetings

5. **Output the risk matrix**
   - Use `file_write` to save the risk register
   - Use `memory_store` to persist risk data for ongoing monitoring

## Output Format

```markdown
# Risk Register: [Project Name]

## Risk Matrix Overview
| | Low Impact (1-2) | Med Impact (3) | High Impact (4-5) |
|---|---|---|---|
| **High Prob (4-5)** | Medium | High | High |
| **Med Prob (3)** | Low | Medium | High |
| **Low Prob (1-2)** | Low | Low | Medium |

## Identified Risks

| ID | Risk Description | Category | Impact | Probability | Score | Strategy | Mitigation | Owner | Status |
|----|-----------------|----------|--------|-------------|-------|----------|------------|-------|--------|
| RSK-001 | [Description] | Technical | 4 | 3 | 12 | Mitigate | [Action] | [Name] | Active |
| RSK-002 | [Description] | Schedule | 5 | 2 | 10 | Accept | Monitor trigger | [Name] | Watching |
| RSK-003 | [Description] | Resource | 3 | 4 | 12 | Transfer | [Action] | [Name] | Mitigating |

## Top 3 Risks Requiring Attention
1. **RSK-001**: [Why this risk is critical and what's being done]
2. **RSK-00X**: [Why this risk is critical and what's being done]
3. **RSK-00Y**: [Why this risk is critical and what's being done]

## Risk Triggers & Escalation
| Risk | Trigger | Escalation Action |
|------|---------|-------------------|
| RSK-001 | [Observable indicator] | [Who to notify, what to do] |
```

## Notes

- Focus on risks that are specific and actionable — vague risks ("something might go wrong") are not useful
- Update the risk register regularly — risks evolve as the project progresses
- Don't confuse risks (uncertain future events) with issues (events that have already occurred)
- Use `memory_store` to track how risks actually materialized in past projects — this improves future identification
- Celebrate risks that were successfully mitigated — this reinforces proactive risk management behavior

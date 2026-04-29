---
name: code-review
description: Perform a thorough code review covering correctness, security, performance, and maintainability
version: "1.0.0"
author: developer
triggers:
  - review
  - code review
  - 审查代码
  - PR review
tool_deps:
  - file_read
  - memory_recall
---

# Code Review Skill

## Execution Steps

1. **Understand the change scope**
   - Use `file_read` to examine the changed files and their surrounding context
   - Identify the type of change: bug fix, feature, refactor, or configuration
   - Recall any project-specific conventions with `memory_recall`

2. **Check for logical errors**
   - Verify that the code does what the commit message or PR description claims
   - Trace the data flow through the changed functions — are all branches reachable?
   - Check edge cases: null/empty inputs, boundary values, error paths
   - Verify that error handling is complete (no silently swallowed errors)

3. **Check for security vulnerabilities**
   - Input validation: are all external inputs sanitized?
   - Injection risks: SQL injection, command injection, XSS, path traversal
   - Data exposure: are sensitive values (API keys, tokens) logged or leaked?
   - Permission checks: are access controls enforced at the right layer?

4. **Check for performance issues**
   - Algorithmic complexity: any O(n²) or worse where O(n) is achievable?
   - Unnecessary allocations: repeated string formatting, cloning where borrowing works
   - Missing caching: repeated computation of the same result
   - Resource leaks: unclosed file handles, connections, or missing cleanup

5. **Check for maintainability**
   - Naming: do variable and function names convey intent clearly?
   - Function length: are functions doing one thing, or should they be decomposed?
   - Code duplication: is the same logic repeated that could be extracted?
   - Comments: are complex decisions explained? Is the "why" documented, not just the "what"?

6. **Output the review report**

## Output Format

```markdown
## Code Review Report

### Summary
[Brief overall assessment: approve / request changes / block]

### Critical Issues (Must Fix)
- [ ] **[File:Line]** Description of critical issue

### Important Issues (Should Fix)
- [ ] **[File:Line]** Description of important issue

### Suggestions (Nice to Have)
- [ ] **[File:Line]** Suggestion description

### Positive Observations
- Good patterns or decisions worth highlighting
```

## Notes

- Always start with `memory_recall` to check for project-specific coding standards or past review feedback
- Prioritize correctness and security over style preferences
- When suggesting alternatives, provide concrete code examples
- If the change is too large for a single review, suggest splitting it into smaller PRs
- Acknowledge good patterns and decisions — reviews should not be purely negative

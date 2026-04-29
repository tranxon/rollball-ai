---
name: debugging
description: Systematically debug and troubleshoot issues with root-cause analysis and verified fixes
version: "1.0.0"
author: developer
triggers:
  - debug
  - fix bug
  - 排查问题
  - troubleshoot
  - investigate
tool_deps:
  - file_read
  - file_write
  - shell
  - memory_recall
---

# Debugging Skill

## Execution Steps

1. **Reproduce the issue**
   - Use `file_read` to examine the relevant source files
   - Use `shell` to run reproduction commands and capture output
   - Document the exact steps to reproduce: environment, inputs, expected vs. actual behavior
   - If the issue is intermittent, document frequency and conditions under which it occurs
   - Use `memory_recall` to check if a similar issue has been debugged before

2. **Analyze logs and symptoms**
   - Use `shell` to examine log files, stack traces, and error messages
   - Identify the failure point: where does execution deviate from expected behavior?
   - Look for patterns: does the error correlate with specific inputs, timing, or load?
   - Check recent changes: `git log --oneline -20` to identify recent commits that may be related
   - Examine configuration: are environment variables and config files set correctly?

3. **Locate the root cause**
   - Form a specific hypothesis: "The bug is in function X because condition Y is not checked"
   - Verify the hypothesis with targeted tests: add logging, use `shell` to run focused tests
   - If the hypothesis is wrong, form a new one — avoid confirmation bias
   - Use binary search: comment out half the code, see if the bug persists, narrow the scope
   - Trace the data flow from input to failure point — where does the data become incorrect?

4. **Implement the fix**
   - Make the minimal change that fixes the root cause — resist the urge to refactor nearby code
   - Use `file_write` to apply the fix
   - Ensure the fix handles all edge cases identified during analysis
   - Add defensive checks if the root cause was a missing validation

5. **Verify the fix**
   - Use `shell` to run the reproduction steps — confirm the issue is resolved
   - Run existing tests to ensure no regressions: `cargo test`, `npm test`, etc.
   - Add a regression test that would have caught the original bug
   - Test edge cases related to the fix
   - If possible, test under conditions that previously triggered the intermittent failure

## Output Format

```markdown
## Debug Report

### Issue
[Brief description of the bug or problem]

### Root Cause
[Precise explanation of why the bug occurs]

### Reproduction
1. Step one
2. Step two
3. Observe: [actual behavior] instead of [expected behavior]

### Fix
[Description of the change made, with file and line references]

### Verification
- [x] Original issue no longer reproduces
- [x] Existing tests pass
- [x] Regression test added: [test name]
- [ ] Edge cases tested: [list any remaining]
```

## Notes

- Never guess — always verify hypotheses before implementing a fix
- If the root cause is unclear after 3 hypothesis iterations, escalate: list what you've tried and what remains unknown
- Keep a debug log: record each hypothesis, test, and result — this prevents going in circles
- Use `memory_store` to persist debugging insights, especially for non-obvious root causes that may recur
- When the bug involves async/concurrent code, pay special attention to race conditions and ordering assumptions

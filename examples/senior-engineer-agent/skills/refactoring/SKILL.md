---
name: refactoring
description: Identify code smells and apply structured refactoring with regression safety
version: "1.0.0"
author: developer
triggers:
  - refactor
  - 重构
  - clean code
  - restructure
  - improve code
tool_deps:
  - file_read
  - file_write
  - shell
  - memory_recall
---

# Refactoring Skill

## Execution Steps

1. **Identify code smells**
   - Use `file_read` to examine the target code
   - Use `memory_recall` to retrieve project-specific conventions and past refactoring decisions
   - Catalog the smells present using the standard taxonomy:
     - **Long Method**: Function exceeds 30 lines or has multiple levels of abstraction
     - **Large Class/Module**: Module handles too many responsibilities
     - **Duplicated Code**: Similar logic exists in multiple places
     - **Feature Envy**: Method uses more data from another class than its own
     - **Shotgun Surgery**: One change requires edits across many files
     - **Magic Numbers/Strings**: Unnamed constants scattered in code
     - **Dead Code**: Unreachable or unused code paths
     - **Premature Abstraction**: Over-engineered generics or traits for current needs
   - Prioritize smells by impact: which ones cause the most bugs, confusion, or slowdowns?

2. **Design the refactoring plan**
   - Choose the appropriate refactoring technique for each smell:
     - Extract Function / Extract Method for Long Method
     - Move Method for Feature Envy
     - Replace Conditional with Polymorphism for complex conditionals
     - Introduce Parameter Object for long parameter lists
     - Replace Magic Number with Named Constant
   - Determine the order of refactoring steps — some refactorings unlock others
   - Estimate risk: will this refactoring change public APIs or behavior?
   - Ensure test coverage exists before starting — add characterization tests if needed

3. **Apply refactoring incrementally**
   - Make one refactoring change at a time — never batch multiple refactorings
   - Use `file_write` to apply each change
   - After each change, use `shell` to run the test suite: `cargo test`, `npm test`, etc.
   - If tests fail after a refactoring step, revert and reassess the approach
   - Commit or checkpoint after each successful refactoring step

4. **Run regression tests**
   - Execute the full test suite with `shell`
   - Run linter and type checker: `cargo clippy`, `tsc --noEmit`, etc.
   - Compare test results before and after — no previously passing test should now fail
   - If the project has integration tests, run those as well
   - Check for performance regressions if the refactoring affects hot paths

5. **Update documentation**
   - Use `file_write` to update any affected documentation
   - Update inline comments that reference old structure or naming
   - Update module-level documentation (rustdoc `///`, JSDoc, docstrings)
   - If public APIs changed, update API documentation
   - Use `memory_store` to record the refactoring rationale for future reference

## Output Format

```markdown
## Refactoring Report

### Code Smells Identified
| # | Smell | Location | Severity | Technique |
|---|-------|----------|----------|-----------|
| 1 | Long Method | src/module.rs:42 | High | Extract Function |
| 2 | Duplicated Code | src/a.rs:10, src/b.rs:25 | Medium | Extract Shared Function |

### Changes Applied
1. **[Step 1]** Extracted `validate_input()` from `process_data()` in `src/module.rs`
   - Tests: PASS (12/12)
2. **[Step 2]** Moved `format_output()` to `OutputFormatter` module
   - Tests: PASS (12/12)

### Verification
- [x] All existing tests pass
- [x] Linter passes with no new warnings
- [x] No behavioral changes introduced
- [x] Documentation updated
```

## Notes

- **Never refactor and fix bugs in the same commit** — keep concerns separate
- If test coverage is low, add characterization tests BEFORE refactoring (test current behavior, even if it seems wrong)
- The safest refactorings are those that the compiler can verify: type-safe renames, trait extractions, etc.
- When in doubt, make the refactoring smaller — you can always do another pass later
- Refactoring should not change observable behavior — if it does, it's a redesign, not a refactor

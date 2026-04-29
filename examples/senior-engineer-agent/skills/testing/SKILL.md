---
name: testing
description: Design and write comprehensive tests including unit, integration, and edge case coverage
version: "1.0.0"
author: developer
triggers:
  - test
  - write tests
  - 编写测试
  - unit test
  - integration test
tool_deps:
  - file_read
  - file_write
  - shell
  - memory_recall
---

# Testing Skill

## Execution Steps

1. **Analyze the test target**
   - Use `file_read` to examine the code that needs tests
   - Identify the public interface: what functions, methods, or APIs need testing?
   - Understand the contract: what are the valid inputs, outputs, and error conditions?
   - Use `memory_recall` to retrieve project-specific testing conventions and patterns
   - Check for existing tests — avoid duplicating coverage

2. **Design test cases**
   - **Happy path**: Verify correct behavior with valid inputs
   - **Boundary values**: Test at the edges of valid ranges (zero, max, empty, single element)
   - **Error paths**: Verify correct error types and messages for invalid inputs
   - **Edge cases**: Concurrent access, large inputs, missing optional fields, encoding issues
   - **Regression tests**: Cover known bugs that have been fixed
   - Prioritize test cases by risk: which failures would be most impactful?

3. **Write unit tests**
   - Use `file_write` to create test files following project conventions:
     - Rust: `#[cfg(test)] mod tests { }` in the same file, or `tests/` directory
     - TypeScript: `*.test.ts` or `*.spec.ts` alongside source
     - Python: `test_*.py` in the same package
   - Follow the Arrange-Act-Assert pattern
   - Each test should test one behavior — avoid assertion-heavy tests that hide failures
   - Use descriptive test names: `test_parse_returns_error_on_empty_input` not `test_parse_1`
   - Mock external dependencies — tests should be deterministic and fast

4. **Write integration tests**
   - Test the interaction between multiple modules or components
   - Use realistic test fixtures and sample data
   - Test the full request-response cycle for APIs
   - Verify side effects: database writes, file system changes, message delivery
   - Integration tests should be in a separate directory from unit tests

5. **Run and verify**
   - Use `shell` to execute the test suite: `cargo test`, `npm test`, `pytest`, etc.
   - Verify that all new tests pass
   - Verify that no existing tests break
   - Check test output for any panics, flaky behavior, or warnings
   - If the project supports coverage reporting, generate a coverage report

6. **Coverage report**
   - Identify untested code paths from coverage data
   - Prioritize additional tests for uncovered critical paths
   - Document the achieved coverage level and any known gaps

## Output Format

```markdown
## Test Report

### Test Target
[Description of what was tested]

### Test Cases Designed
| # | Category | Test Name | Covers |
|---|----------|-----------|--------|
| 1 | Happy Path | test_successful_parse | Valid JSON input |
| 2 | Boundary | test_empty_array_input | Empty array handling |
| 3 | Error | test_invalid_format_error | Malformed input rejection |

### Files Created
- `src/module.rs` (added unit tests inline)
- `tests/integration_module.rs` (integration tests)

### Results
- Unit tests: X passed, 0 failed
- Integration tests: Y passed, 0 failed
- Coverage: Z% (target: >= 80%)

### Known Gaps
- [ ] Async error paths not covered (requires mock runtime setup)
- [ ] Concurrency edge case pending (needs stress test harness)
```

## Notes

- Tests are first-class code — apply the same quality standards as production code
- Prefer explicit assertions over error-swallowing patterns (`unwrap()` in Rust tests is fine, but `expect("reason")` is better)
- Don't test implementation details — test behavior and contracts
- If a test is flaky, fix it immediately — flaky tests erode trust in the entire suite
- Use `memory_store` to record discovered edge cases and testing patterns for future reference

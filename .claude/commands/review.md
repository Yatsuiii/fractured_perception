# Command: /review

## Analyze Rust code for

* Performance issues (unnecessary allocations, hot-path cloning)
* Ownership/borrowing problems
* Code clarity and module responsibility
* Maintainability (function length, coupling between systems)
* Perception correctness (each role's view must be fully distinct)
* Error handling (no unwrap() in core systems)

## Output

* Issues found (with file:line references)
* Improvements (prioritized by impact)
* Suggested refactor (module splits if file exceeds ~300 lines)

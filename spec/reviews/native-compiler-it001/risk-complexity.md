---
id: SR-029
title: "Risk-complexity review of native compiler IT-001 intake"
type: SpecReview
analysis: risk-complexity
scope: "IT-001, FR-001, FR-002, TC-001, Task-009"
review_set: all
---
## Summary

Native artifact admission is high technical risk because acceptance depends on
exact cross-repository identities, parser-free linking, and lossless numeric
endpoints; it is also high volatility while the producer traceability gate is
open. The fixture lane should remain narrowly pinned and fail closed rather
than add compatibility behavior.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The producer ELF digest is intentionally rebuild-volatile while PR72 is open; pinning that seal would turn a valid regenerated fixture into a false failure, whereas accepting it without independently derived selectors would enable artifact self-authorization. | IT-001 Notes; PR72; Task-009 | correct-requirement-no-evidence |

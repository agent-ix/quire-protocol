---
id: SR-028
title: "Evidence review of native compiler IT-001 intake"
type: SpecReview
analysis: evidence
scope: "IT-001, FR-001, FR-002, TC-001, TM-001, Task-009"
review_set: all
---
## Summary

The public API fixture lane is the appropriate integration method for the
admission, identity, numeric, and refusal obligations. The deterministic
advisor could not run because its Quire-version detection is inconsistent with
the installed `quire 0.31.0`; the direct coverage report additionally exposes
the missing FR criterion bindings.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | `quoin advise --json` fails before producing recommendations despite installed `quire 0.31.0`, reporting that it cannot determine a supported Quire version; evidence-method advice is therefore unavailable and must not be claimed. | IT-001; TC-001; TM-001 | correct-requirement-no-evidence |
| FND-002 | medium | Criterion-level bindings now back four native-intake rows, while `Task-009` remains intentionally untracked because it is a plan task rather than a matrix target; do not treat its tag as coverage credit. | TC-001; FR-001; FR-002; TM-001 | correct-requirement-no-evidence |

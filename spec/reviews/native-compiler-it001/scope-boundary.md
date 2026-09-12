---
id: SR-030
title: "Scope-boundary review of native compiler IT-001 intake"
type: SpecReview
analysis: scope-boundary
scope: "IT-001, FR-001, FR-002, Task-009"
review_set: all
---
## Summary

The contract correctly assigns native grammar, compiler emission, numeric-wire
semantics, and canonicalization to quire-spec-language; this crate consumes the
public reader and validates independently selected identities. The consumer
still needs evidence that its dependency path contains only that reader, not a
locally duplicated frontend.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | SC-05 names the required ownership boundary but lacks an implemented positive source/dependency inventory that identifies `quire_spec_language::protocol_artifact::read` as the sole native-reader entry point. | IT-001-SC-05; FR-001; FR-002; Task-009 | correct-requirement-no-evidence |

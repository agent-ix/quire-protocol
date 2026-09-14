---
id: SR-060
title: "Base review of the published version 2 handoff closeout"
type: SpecReview
analysis: base
scope: "IT-001 SC-06..SC-09, Task-012, TM-001, and the version 2 handoff tests at qprotocol e53fb55 against qspec-language 93c411b"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-012, type: reviews }
---
## Summary

The producer dependency is no longer blocked: `quire-spec-language` PR #92
exports the handoff directory, every fixed member name, and the mutation-manifest
format at `93c411b287ccf4789879ac4ef3094fc921dada58`. The closeout remains a
consumer implementation task, not an architecture change. Three correctable
gaps were found in the pre-closeout state and are now explicit in Task-012.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The default lane reports `7 passed; 6 ignored`; every test that submits A's real handoff is ignored behind `QUIRE_PROTOCOL_A_V2_HANDOFF`, even though A now exports `PUBLISHED_HANDOFF`. SC-06 through SC-09 therefore remain unexecuted until the consumer removes the environment gate and ignores. | IT-001 Notes; Task-012 Remaining; `tests/support/v2_handoff/mod.rs`; `tests/it001_v2_handoff.rs` |
| FND-002 | high | Task-012 says SC-09 runs, but no current test measures the admitted report and probes every declared dimension at exact usage, one-short usage, and fresh exact usage. The plan now names that missing test explicitly over `protocol_artifact::Dimension::ALL`. | IT-001-SC-09; Task-012 Remaining; `tests/it001_v2_handoff.rs` |
| FND-003 | medium | The old B-authored scenario index remains beside A's published 28-case manifest. Keeping both would preserve two competing sources of version 2 vocabulary. Task-012 now requires deletion of `tests/support/v2/mod.rs` and `tests/it001_v2_intake.rs` and reconciliation of current tracking while retaining historical reviews. | Task-012 Remaining; FR-013; FR-014 |
| FND-004 | medium | The current executing suite does not visibly submit both contract versions to the opposite public entry point. The closeout must make those calls directly; source inspection or a manifest label alone is not scenario evidence. | IT-001-SC-06; FR-013-AC-4; Task-012 Remaining |

## Disposition

All findings are in scope for issue #48 and have concrete implementation or
tracking steps in Task-012. No new requirement, subsystem, or producer work is
needed. The ticket is ready for TDD implementation after dependency and EARS
self-review.

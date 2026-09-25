---
id: SR-127
title: "Base review of the QSL-250 producer-run handoff spec amendments"
type: SpecReview
analysis: base
scope: "IT-001 SC-01, SC-06 procedure, Inputs, Dependencies and Notes; spec/tests.md IT-001 rows; ADR-007 rejected alternatives; DEPENDENCIES.md, against tests/it001_native_handoff.rs and tests/support/v2_handoff/qsl_producer.rs on branch qsl-250-bytedigest-import"
review_set: base
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/ADR-007, type: reviews }
---
## Summary

The amended IT-001 text now matches the code. Both lanes read A's selection
sidecar before the offer and check every source, model and dependency byte
against its digests. The version 1 lane also re-admits the model through A's
reader. Both lanes get their complete handoff from a fresh run of A's producer
examples at the pinned revision. Reviewing the amendment found one accepted ADR
and three IT-001 passages that still described a B-retained selection. They are
fixed in the same change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | ADR-007's rejected alternative refused "A's emitted sidecar as the selection" and quoted AD-001 for it. AD-001 contains no such sentence. The version 2 lane had already read `expected-v2.json` since Task-012, and the version 1 "independent" selection was a stale copy of A's `expected.json`. Fixed: the alternative now rejects a selection read out of the offered bytes, and describes the sidecar as a separately published member that B reads before the offer and digest-checks. | ADR-007 Alternatives; IT-001 SC-01; `tests/it001_native_handoff.rs` |
| FND-002 | medium | IT-001 test procedure step 1, step 6, the Inputs section and the version 2 surface paragraph still said "B-selected retained" or "independently retained" selection. Fixed to name A's selection sidecar and the digest-checked original inputs. | IT-001 Test Procedure 1 and 6; IT-001 Inputs |
| FND-003 | low | The claim that B checks every dependency byte against the sidecar's digests was only true through A's reader in the version 1 lane. Fixed: `Fixture::load` now asserts each dependency file's digest directly. | IT-001 SC-01; `tests/it001_native_handoff.rs` |
| FND-004 | low | `spec/tests.md` fails structural validation at the functional-coverage table header ("Coverage Status" vs "Status"). This predates the change and is out of scope, so it is recorded here and not fixed. | `spec/tests.md` Functional Coverage |

## Disposition

FND-001 to FND-003 are fixed in this change. FND-004 predates it and is not
addressed here. `quire validate` reports no errors in IT-001 or ADR-007.

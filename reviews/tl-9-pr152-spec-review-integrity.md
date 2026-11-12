---
id: SR-2127063
title: "TL-9 PR #152: spec-review integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-protocol; PR #152 head c492bdb; spec/functional/FR-019, spec/functional/FR-202, spec/test-cases/TC-202"
review_set: subset
date: 2026-11-12
---

## Summary

Ticket: TL-9. Method: spec-review/spec-integrity-analysis. PR agent-ix/quire-protocol#152 head c492bdb; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-300 -> FR-479 rename rewrote 'QSpec main `316743b` adds FR-300' to '... adds FR-479', which is now a false provenance statement (that commit added the id FR-300). Per repo CLAUDE.md, a touched SHA reference should be removed, not edited; drop the commit id. | spec/functional/FR-019-derive-obligation-activation-state.md:114 |
| FND-002 | medium | The rename is partial: FR-004 (including a `depends_on` relationship to ix://agent-ix/quire-specification/FR-300), FR-101, AD-001 and IT-002 still cite FR-300 as the activation binding, and quire-specification FR-300 is now 'expose-rust-lifecycle-api'. Those edges now resolve to the wrong artifact. Pre-existing and outside the diff, but this PR started the rename; finish it here or ticket it. | spec/functional/FR-004-assess-finite-global-conformance.md:34 |

## Verdict

PASS WITH FINDINGS. The FR-202/TC-202 change is correct against quire-specification main: the FR-489 Settlement table now reads 'Work or resource bound reached -> `failed`, FR-241 `resource-incomplete` (FR-331 result `incomplete`), basis `unavailable`', and FR-360's `failed` row admits execution `failed` or `resource-incomplete`, so dropping the `incomplete`-as-label open question is justified and AC-5/TC-202 agree. FR-479 is quire-specification's obligation-activation-binding, so FR-019's retarget is correct.

Units examined:

- `FR-019` (examined)
- `FR-202` (examined)
- `FR-202-AC-5` (examined)
- `TC-202` (examined)

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The mechanical FR-300 -> FR-479 sweep changed 'the old FR-300 ... contract is retired' into 'the old FR-479 ... contract is retired', which now says the live binding FR-019 and FR-004 depend_on is retired. Reword (e.g. 'the former producer-supplied activation-binding contract is retired; Protocol derives the FR-479 binding itself'). | spec/integration/IT-002-consume-observation-temporal-facts.md:130 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 6eb955c: QSpec adds FR-479 and closes the event-triggered bridge: an activation binding is the exact source for a dynamic FR-230 instance. |
| FND-002 | fixed | 6eb955c: FR-004: - target: ix://agent-ix/quire-specification/FR-479 / type: depends_on; AD-001: '----> FR-479 activation binding'; FR-101: 'supplied through the FR-479 binding and FR-045'; no FR-300 reference remains outside FR-300's own file. |

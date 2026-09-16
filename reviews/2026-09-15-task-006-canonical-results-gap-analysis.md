---
id: SR-117
title: "Gap analysis of the Task-006 atomic canonical-result orchestration"
type: SpecReview
analysis: gap-analysis
scope: "Task-006 / quire-protocol#8, FR-006/FR-008/FR-031..FR-033/FR-036..FR-040, TC-006/TC-011, Plan-001 and TM-001"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-011, type: reviews }
---
# SR-117: Gap Analysis of the Task-006 Atomic Canonical-Result Orchestration

## Summary

Task-006 now composes exact activation-derived conformance and sealed claim
dispatch into privately produced, strictly read canonical rows before one
AssessmentBundle commit. The same operation preserves stage accounting and
routes typed refusals. Task, plan and matrix now record that the public owner
surfaces cannot yet provide authenticated oracle/dependence support or
correction inputs; the fail-closed rows are not pre-credited as completion.

## Verdict

**BLOCKED for Task-006 / quire-protocol#8.** Local public-construction and
activation-selection gaps are remediated. FR-032 authenticated support and
FR-033 applicable correction remain ungrounded in the pinned owner APIs. This
is not a PASS for Task-007, Task-008, Task-010, Plan-001 or epic #14.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: stage outputs could be retargeted because result construction accepted a caller-selected dispatch report and restated owner facts. Atomic Engine orchestration now retains or derives every stage-owned value. | FR-006; TC-006 | correct-requirement-no-evidence |
| FND-002 | high | Resolved: the raw canonical producer was a public bypass around the Task-006 commit boundary. It is crate-private and its external import is compile-fail. | FR-006-AC-1/9; TC-006 | implementation-bug-despite-evidence |
| FND-003 | medium | Resolved: model/runtime provenance was substitutable and the runtime-reference carrier was not connected to production assembly. Membership checks and strict carrier round-trips now execute on the Engine path. | FR-031; TC-006 | correct-requirement-no-evidence |
| FND-004 | medium | Resolved: FR-008-AC-9 had Inspection in the matrix but no executable evidence. TC-011 now verifies every parent `contains` edge, child `part_of` edge, unique Behavior allocation and AC-1..AC-8 child criterion reference. | FR-008-AC-9; TC-011 | correct-requirement-no-evidence |
| FND-005 | medium | Resolved: Task-006 status and FR/TC matrix rows remained in-progress after the executable surface closed. Task, plan, index, chunk evidence, requirement status text and matrix now agree with the candidate boundary. | Task-006; Plan-001; TM-001 | correct-requirement-no-evidence |
| FND-006 | high | Resolved by independent PR-head review: a remaining public result-facts aggregate allowed caller substitution for values the requirement assigns to preceding typed stages. Only reproduction and lowered owner ceilings remain caller-controlled; semantic result facts are private to the Engine transition, and correction stays absent until an owner-authenticated contradiction capability exists. | FR-006 Inputs/Behavior; FR-006-AC-1; TC-006 | implementation-bug-despite-evidence |
| FND-007 | high | Resolved by independent PR-head review: result-owner limit exhaustion occupied the claim-refusal branch despite the specified non-interchangeable resource outcome. Owner limits now terminate the operation as `OperationOutcome::ResourceIncomplete` before bundle commit. | FR-006-AC-9; NFR-002; TC-006 | implementation-bug-despite-evidence |
| FND-008 | medium | Resolved by independent PR-head review: raw public constructors and order-insensitive dispatch validation left alternate paths around the shared envelope and ordered request contract. Public startup is enveloped and the requested population must match in exact order. | FR-006-AC-9; TC-006/TC-011 | implementation-bug-despite-evidence |
| FND-009 | medium | Resolved during remediation: a checked-predicate view could be valid under the right package while naming another declaration or an unreachable leaf. Result startup now proves exact request/declaration/subject correspondence and exact membership in the temporal subject's QSL-published reachable leaf set. | FR-006-AC-1/4; FR-031-AC-5; TC-006 | implementation-bug-despite-evidence |
| FND-010 | high | Resolved by final independent review: activation facts were not bound to the result Engine's exact activation. Definition, QObs subject and strict QSL request bytes must now match before conformance. | FR-006-AC-1/4; TC-006 | implementation-bug-despite-evidence |
| FND-011 | high | Open owner-capability blocker: orchestration had invented decisive support and used obligation identity as oracle identity. Fabrication is removed and the row fails closed, but no pinned owner capability supplies authenticated oracle/dependence facts, so FR-032 is not complete. | FR-032-AC-2; FR-033-AC-3; TC-006 | missing-owner-capability |
| FND-012 | high | Resolved by final independent review: participation invented and sorted local role-instance strings. It now retains only exact bound FR-287 role-instance subjects in their canonical key order or emits a typed owner-evidence refusal. | FR-006; FR-032; TC-006 | implementation-bug-despite-evidence |
| FND-013 | high | Open owner-capability blocker: correction controls allowed caller-authored canonical semantics. The fabrication seam is removed, but applicable correction awaits an owner-authenticated contradiction/correction capability and has no public Engine evidence. | FR-006; FR-033; TC-006 | missing-owner-capability |
| FND-014 | high | Resolved by final independent review: unrelated assembly failures collapsed to one envelope identity. Private exhaustive variants now have distinct registered codes, axes, typed causes and affected identities, checked by an injectivity test. | FR-006-AC-9; FR-008; FR-014; TC-006/TC-011 | implementation-bug-despite-evidence |
| FND-015 | high | Resolved by final independent review: owner limits were reconstructed after refusal using raw limits and fabricated sites. Producer output/string/depth/field work now charges the shared accountant at distinct reconciled sites under effective owner ceilings. | FR-006-AC-9; NFR-002; TC-006/TC-009 | implementation-bug-despite-evidence |

## Reconciliation

| Obligation | Evidence | Result |
| --- | --- | --- |
| Atomic result production | Public Engine performs conformance, dispatch, private produce, strict read and bundle commit with no foreign-report seam | Complete |
| Orthogonal owner facts | Exact axes/completeness/global closure are reused. Activation and exact role-instance participation derive from owner bindings. Current owner outputs carry no exact oracle/dependence capability, so truth/settlement are explicitly unavailable and support empty rather than invented. | Partial; blocked on owner capability |
| Runtime provenance | The unique selected model and exact activated, bound input are retained; reversible runtime-reference carrier mutations execute | Complete |
| Refusal routing | Current public failure routes retain typed stage/domain/prerequisite/cause/partial-output/resource fields; row-assembly interiors are injective and producer owner limits route through exact accountant sites | Complete |
| Test Matrix | TC-006 covers the available atomic/fail-closed surface; authenticated support and applicable public correction scenarios remain unexecuted. FR-008-owned TC-011 is implemented. | Partial |
| Tracking | Task-006, Plan-001 and the matrix identify the owner blockers without claiming the candidate complete. | Complete |

## Coverage and Boundary

The structural audit found that prior Task/matrix/review status had pre-credited
safe absence as completed semantics. Those records are now partial. Searches
over the changed surface found no `todo!`, `unimplemented!`, ignored Task-006
test, placeholder success or new unsafe/concurrency path; that local hygiene
does not discharge the two missing owner capabilities recorded above.

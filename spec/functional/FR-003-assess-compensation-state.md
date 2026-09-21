---
id: FR-003
title: "Assess compensation state"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-002
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-019
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-020
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-021
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-022
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-023
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-024
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-025
    type: contains
  - target: ix://agent-ix/quire-specification/FR-055
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-058
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-230
    type: references
  - target: ix://agent-ix/quire-specification/FR-231
    type: references
  - target: ix://agent-ix/quire-specification/FR-232
    type: references
  - target: ix://agent-ix/quire-specification/FR-233
    type: references
  - target: ix://agent-ix/quire-specification/FR-234
    type: references
  - target: ix://agent-ix/quire-specification/FR-236
    type: references
  - target: ix://agent-ix/quire-specification/FR-239
    type: references
  - target: ix://agent-ix/quire-specification/FR-264
    type: references
---
# FR-003: Assess compensation state

## Description

When protocol observations establish activation, effect, commit, retry or recovery facts, the evaluator SHALL derive compensation state only through the atomic requirements FR-019 through FR-025 that together discharge source requirements FR-055 through FR-058.
When any part of compensation state is derived, the evaluator SHALL invoke no business operation and no compensating operation.
When any atomic requirement FR-019 through FR-025 does not produce its positive fact, the evaluator SHALL preserve that requirement's exact outcome: missing admitted evidence is pending/unavailable, invalid or contradictory identity is a typed refusal, an unimplemented admitted relation is unsupported, budget exhaustion is resource-incomplete, and an established protocol breach is a violation.
The evaluator SHALL NOT collapse those outcomes into aggregate success or one generic unavailable state.

## Inputs

A constructor-private version-2 linked protocol and its validated runtime
bindings; exact trigger and effect observations; immutable captures and
evaluation anchors; authored retry/commit/recovery relations; and exact
progress/population/completeness authorities. Production callers SHALL NOT
construct compensation definition, workflow, role, attempt or effect identities
independently of the linked graph and imported FR-287 values.

## Outputs

Typed activation, registration, attempt, effect, commit and recovery facts with
exact violations, pending/unavailable dependencies, unsupported dispositions,
resource-incomplete outcomes or refusals.

## Behavior

This requirement is the aggregate. It establishes no obligation of its own beyond
composition: each obligation it previously stated in prose is now stated, once
and independently verifiably, by exactly one of its parts.

| Part | Obligation it owns |
| --- | --- |
| [FR-019](./FR-019-derive-obligation-activation-state.md) | Activation state derivation, identity-substitution refusal, activation independent from truth. |
| [FR-020](./FR-020-register-compensation-once-per-effect.md) | Registration only after the paired successful effect, and one registration per effect identity under replay. |
| [FR-021](./FR-021-distinguish-commit-ordering-violations.md) | The three distinct commit-ordering violations. |
| [FR-022](./FR-022-enforce-finite-compensation-retry-bound.md) | The authored finite retry bound and its refusal. |
| [FR-023](./FR-023-preserve-attempt-and-effect-distinctness.md) | Attempt, effect and outcome distinctness and identity retention. |
| [FR-024](./FR-024-decide-recovery-only-by-authored-relation.md) | Full and partial recovery decided only by the authored relation, with population completeness. |
| [FR-025](./FR-025-assess-compensation-without-io.md) | The I/O purity boundary over the whole assessment surface. |

The facts remain separate facts. Attempt, effect, commit and recovery facts are
reported side by side and no one of them is derived from another; a successful
send, receive, delivery, attempt or operation invocation establishes neither a
successful business effect nor a recovery relation. That separation is the reason
the parts are separate requirements rather than one.

The evaluator SHALL compute every registry and ledger change into a private
candidate and commit it only after the complete request succeeds. Exact replay
is idempotent. A foreign identity, conflicting replay, exceeded retry,
unsupported construct or resource-incomplete outcome SHALL leave all prior
state unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-003-AC-1 | Activation over the whole assessment is exactly what [FR-019](./FR-019-derive-obligation-activation-state.md) reports: triggered, inactive and activation-unknown for established, absent-under-complete-closure and unavailable-under-open/incomplete triggers, with identity substitution refusing activation, and no other path assigns an activation state. | Test (TC-003) |
| FR-003-AC-2 | Registration over the whole assessment is exactly what [FR-020](./FR-020-register-compensation-once-per-effect.md) creates: one registration per paired successful effect, none from sends, receives, deliveries or failed attempts, and none added by replay of the same effect identity. | Test (TC-003) |
| FR-003-AC-3 | The ordering violations of [FR-021](./FR-021-distinguish-commit-ordering-violations.md) and the retry-bound refusal of [FR-022](./FR-022-enforce-finite-compensation-retry-bound.md) are reported as four mutually distinct violations, and no aggregation merges any two of them. | Test (TC-003) |
| FR-003-AC-4 | The attempt, effect and outcome facts of [FR-023](./FR-023-preserve-attempt-and-effect-distinctness.md) reach the aggregate result with their exact attempt, effect and registration identities intact and no fact collapsed into another. | Test (TC-003) |
| FR-003-AC-5 | Recovery in the aggregate result is exactly what [FR-024](./FR-024-decide-recovery-only-by-authored-relation.md) decides, so a partial success alongside a full-restoration failure is reported as both, and no aggregate path reports recovery success the authored relation did not produce. | Test (TC-003) |
| FR-003-AC-6 | A missing trigger, state, effect, commit, population, membership or completeness authority anywhere in the assessment leaves activation, registration and recovery without success: [FR-019](./FR-019-derive-obligation-activation-state.md)-AC-4, [FR-020](./FR-020-register-compensation-once-per-effect.md)-AC-4, [FR-022](./FR-022-enforce-finite-compensation-retry-bound.md)-AC-4 and [FR-024](./FR-024-decide-recovery-only-by-authored-relation.md)-AC-4 each hold, and the aggregate reports unavailable rather than composing a success from the parts that did hold. | Test (TC-003) |
| FR-003-AC-7 | The purity boundary of [FR-025](./FR-025-assess-compensation-without-io.md) holds over the whole assessment: the implementation performs no network, storage, telemetry, business-operation or compensation-operation I/O while assessing compensation; production construction requires the linked graph/runtime bindings, and every refused or incomplete candidate leaves prior state unchanged. | Test (TC-003, TC-009) |

## Adopted vocabulary

Every noun below is typed upstream and adopted by reference. Each part carries
the slice of this table it actually uses; the table is retained here as the
shared mapping the decomposition was derived from, not as a second definition.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Protocol obligation | `FR-230` | derived, keyed (clause subject, semantic trigger-event identity) |
| Obligation activation state | `FR-231` | value |
| Activation capture | `FR-232` | value |
| Compensation registration | `FR-233` | derived, keyed (source compensation definition identity, established forward effect identity) |
| Compensation obligation lifecycle | `FR-234` | value |
| Compensation attempt outcome | `FR-236` | value |
| Compensation retry policy | `FR-239` | value |
| Membership rule | `FR-264` | value |

`FR-233`'s key is exactly this requirement's replay rule: registration is keyed on
(compensation definition identity, established forward effect identity), so
replaying one effect identity cannot mint a second registration. The rule is
adopted, not restated.

### Accepted owner rulings and remaining gap

QSpec #28/#31/#40 fixes the FR-235 attempt key, FR-237 forbidding-commit
selection, FR-238 relation key, FR-233 registration key, distinct FR-287 effect
identity and FR-231 activation rule. QSpec #57 resolves FR-236 on QSpec main
`478afa2`: `failed`, `timed-out`, `pending`, and `successful` are admissible
in both per-attempt and recovery-relation domains, while `duplicate` is
effect-only. Those value domains are distinct and no attempt outcome implies a
recovery outcome.

## Dependencies

- [FR-002](./FR-002-link-finite-protocol-representation.md).
- [FR-019](./FR-019-derive-obligation-activation-state.md) through
  [FR-025](./FR-025-assess-compensation-without-io.md) are the parts this
  requirement aggregates.
- `ix://agent-ix/quire-specification/FR-055` through `FR-058`.

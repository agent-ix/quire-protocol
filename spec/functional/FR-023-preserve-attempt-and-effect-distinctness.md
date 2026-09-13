---
id: FR-023
title: "Preserve compensation attempt and effect distinctness"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-020
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-058
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-226
    type: references
  - target: ix://agent-ix/quire-specification/FR-233
    type: references
  - target: ix://agent-ix/quire-specification/FR-235
    type: references
  - target: ix://agent-ix/quire-specification/FR-236
    type: references
---
# FR-023: Preserve compensation attempt and effect distinctness

## Description

When several compensation observations relate to one registration, the evaluator SHALL retain each attempt and each established compensation effect as separate subjects with their own identities.
When an attempt outcome is reported, the evaluator SHALL report it as an exact member of the `ix://agent-ix/quire-specification/FR-236` vocabulary.
When a failed attempt is followed by a successful retry, the evaluator SHALL retain both attempts rather than replacing the first.
When a send, a receive, a delivery, an attempt or a compensating-operation invocation succeeds, the evaluator SHALL NOT read that success as an established compensation effect.
When an attempt outcome has not been established, the evaluator SHALL leave it absent rather than defaulting it to `pending`, `failed` or any other member.

## Inputs

A compensation registration typed by
`ix://agent-ix/quire-specification/FR-233`, compensation attempts typed by
`ix://agent-ix/quire-specification/FR-235`, compensation effects typed by
`ix://agent-ix/quire-specification/FR-226`, and per-attempt outcomes drawn from
`ix://agent-ix/quire-specification/FR-236`.

## Outputs

Per-attempt facts and aggregate facts retained separately, each attempt carrying
its own identity, its related effect identity where one was established, and one
exact outcome label or absence.

## Behavior

`FR-058` requires the evaluator to "distinguish a repeated receipt, retried
attempt and duplicate business effect", and `FR-058-AC-2` that "A timed-out
attempt, failed attempt and duplicate effect remain three distinct outcomes".
`FR-058-AC-1` states that "A failed refund attempt followed by one successful
retry retains both attempts and one successful refund effect".

An attempt identity never compares equal to an effect identity. `FR-235-AC-2`
states that rule, and `FR-235-AC-3` refuses "A transport trace identifier,
successful transport status, repeated delivery or equal payload alone" as an
attempt or effect identity.

A `successful` outcome is not a recovery result. `FR-236-AC-4` states that a
`successful` attempt outcome alone does not read as one; recovery is decided only
under [FR-024](./FR-024-decide-recovery-only-by-authored-relation.md).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-023-AC-1 | A failed attempt, a timed-out attempt, a duplicate effect and a successful attempt remain four separately reported facts and none collapses into another. | Test (TC-003) |
| FR-023-AC-2 | A failed attempt followed by one successful retry retains two attempts and one established compensation effect, each with its own retained identity and its owning registration. | Test (TC-003) |
| FR-023-AC-3 | An attempt identity compares unequal to every effect identity, and a transport status, trace identifier, repeated delivery or equal payload establishes neither. | Test (TC-003) |
| FR-023-AC-4 | A successful send, receive, delivery, attempt or compensating-operation invocation establishes no compensation effect. | Test (TC-003) |
| FR-023-AC-5 | An attempt whose outcome is unestablished carries an absent outcome and receives no default member, and every reported outcome matches an exact `FR-236` label with no alias spelling accepted. | Test (TC-003) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Compensation attempt | `FR-235-compensation-attempt` | identity kind not established upstream |
| Compensation attempt outcome | `FR-236-compensation-attempt-outcome` | value |
| Business effect | `FR-226-business-effect` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |
| Compensation registration | `FR-233-compensation-registration` | derived, keyed (source compensation definition identity, established forward effect identity) |

### Where the upstream vocabulary does not yet reach

This requirement's central obligation rests on two identities upstream has not
established, and it cannot be discharged until they exist.

- **Compensation attempt identity.** `FR-235`'s open gap states that the prose
  establishes an attempt has an identity, is distinct from an effect identity,
  and several things that cannot serve as it, and then that "It does not
  establish:" its identity kind, its key components, or an order. AC-2 requires
  two attempts to be retained as two; no stated component separates them.
  `FR-235` records the same failure: "a failed refund attempt and its successful
  retry must both be retained, and no stated component separates them." The
  owner is `agent-ix/quire-specification`.
- **Duplicate-effect detection.** `FR-226` states that its declared key "is
  constant across two effect claims at one static node within one workflow
  instance, so two duplicate effects compare equal under it and the duplicate the
  evaluator is required to detect is undetectable", and `FR-226-AC-5` states that
  "the FR-058 duplicate-effect outcome is not reachable from the key alone".
  AC-1's `duplicate` fact has no key to rest on.
- **Which outcomes are per-attempt and which are aggregate.** `FR-236`'s open gap
  states that FR-058 Outputs lists the five outcomes together "without stating
  which members are admissible per attempt and which are admissible only for the
  aggregate", and that "`duplicate` in particular labels a business effect rather
  than an attempt. No requirement partitions the vocabulary, and this artifact
  does not partition it." AC-1 therefore reports the four facts without asserting
  which level each belongs to.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-020](./FR-020-register-compensation-once-per-effect.md) supplies the owning
  registration.
- [FR-024](./FR-024-decide-recovery-only-by-authored-relation.md) owns the
  recovery decision a successful outcome does not supply.
- `ix://agent-ix/quire-specification/FR-058` establishes the distinctness rules
  and the outcome vocabulary.

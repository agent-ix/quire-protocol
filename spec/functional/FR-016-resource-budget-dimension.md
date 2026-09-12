---
id: FR-016
title: "Resource budget dimension"
type: FR
object: enumeration
relationships:
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: specifies
  - target: ix://agent-ix/quire-protocol/FR-009
    type: specifies
---
# FR-016: Resource budget dimension

## Description

When a request carries a resource budget, the library SHALL account that budget separately on each dimension of this vocabulary rather than against one scalar total.
When consumption on one dimension would exceed the budget granted on that dimension, the library SHALL return that dimension's typed resource-incomplete outcome naming the dimension.

## Values

Each member is one accounted dimension. `NFR-002` enumerates the request
dimensions; `FR-009` "Bounds" adds the three dimensions resolution consumes
before any producer value is read.

| Value | Description |
| --- | --- |
| `input-byte` | Bytes of caller-supplied input decoded or retained. Inclusive maximum; the increment is charged before decoding (NFR-002 Scope). |
| `definition` | Declared definitions admitted. Inclusive count maximum. |
| `node` | Protocol nodes admitted into the linked representation. Inclusive count maximum. |
| `edge` | Edges admitted into the linked representation. Inclusive count maximum. |
| `role` | Roles admitted. Inclusive count maximum. |
| `message` | Messages admitted. Inclusive count maximum. |
| `branch` | Branches explored during finite replay. Inclusive count maximum. |
| `iteration` | Iterations executed during finite replay. Inclusive count maximum. |
| `active-instance` | Concurrently active instances retained. Inclusive count maximum. |
| `observation` | Observation facts consumed. Inclusive count maximum. |
| `lineage` | Support-lineage entries validated. Inclusive count maximum. |
| `result-item` | Items constructed into a canonical result. Inclusive count maximum. |
| `derived-artifact` | Derived artifacts constructed. Inclusive count maximum. |
| `output-byte` | Bytes of constructed output. Inclusive byte maximum. |
| `operation` | Accounted operations performed. Inclusive operation maximum. |
| `accounted-allocation` | Accounted allocations made. Inclusive allocation maximum. |
| `elapsed-time` | Wall-clock progress against a monotonic deadline, checked before and after each declared indivisible unit. Recorded with the observed elapsed time and any indivisible-unit overshoot, and excluded from the reproducibility comparison — see **Elapsed time is recorded, never compared**. |
| `resolution-entry-count` | Manifest entries resolved by FR-009 before any producer value is read. FR-009 "Bounds": resolution consumes a budget "covering the entry count, the total bytes digested, and a deadline". |
| `resolution-bytes-digested` | Bytes digested during FR-009 resolution. |
| `resolution-deadline` | The deadline FR-009 resolution runs under. It is resolution's own deadline dimension and is not the request's `elapsed-time` budget; exceeding it returns resolution's resource-incomplete outcome and reads no producer value. |

Membership is closed to the dimensions the requirements above declare. A
dimension not declared by a requirement has no budget here, and this repository
invents no universal threshold for it (NFR-002 Rationale).

## A budget is per dimension, not scalar

A budget is a vector over these members, not one number. Each dimension is
granted, charged and exhausted independently: exhausting `node` does not consume
`edge`, and a grant on `operation` neither funds nor limits `input-byte`.
`NFR-002` states the charge rule per dimension — the library "checks the
requested increment before allocating, decoding, inserting, or starting the next
unit of work and refuses that unit if it would exceed its maximum" — and
`FR-009-AC-6` states the same per-dimension shape for resolution.

The probe that makes this observable holds the input constant and varies only
the budget, in three positions per dimension:

| Position | Expected disposition |
| --- | --- |
| Exactly sufficient | Admits. The maximum is inclusive, so the exact value is admitted rather than rejected. |
| One unit short | Returns that dimension's typed resource-incomplete outcome, naming that dimension and no other. |
| Fresh budget at the exact value, after the one-short attempt | Admits again, identically to the first position. |

The third position is the load-bearing one: it proves that no budget, partial
consumption or accounted state carries across an exhausted attempt. `NFR-002`
requires zero "work admitted after a limit is exhausted", and `FR-009-AC-6`
requires that "the same manifest under a fresh budget completes".

## Resource-incomplete is not a refusal

A resource-incomplete outcome and a typed refusal are distinct outcomes with
distinct identities. A refusal says the offer was not admissible; a
resource-incomplete outcome says the offer was not decided within the budget
granted, and it asserts nothing about admissibility. The two must not be
collapsed, and a resource-incomplete outcome must not be reported as a refusal
of the input, because doing so would attribute to the producer's bytes a verdict
that belongs to the caller's budget. `FR-008` owns the refusal envelope and
`NFR-002` owns the resource-incomplete outcome; they are separate owners for
separate outcomes.

## Elapsed time is recorded, never compared

`elapsed-time` and `resolution-deadline` are dimensions whose granted and
consumed figures are retained like every other dimension, and whose consumed
figure is **excluded from the reproducibility comparison**. `NFR-005`
Verification states the reason exactly: elapsed time "is recorded but excluded
from the reproducibility comparison, because it is not reproducible and NFR-001
requires the retained evidence to be." Excluding it from the comparison does not
excuse recording it: an omitted elapsed-time figure is a missing figure, not an
excluded one.

## Identity and comparison

- **Identity kind** — *value*, per [FR-011](./FR-011-consumer-identity-discipline.md).
  A dimension label has no identity of its own and is compared componentwise as
  an exact string. Authority is not applicable to the label. The *budget* on a
  dimension is consumer-authored: it is the caller's grant, never a producer's
  assertion, and no producer-emitted value selects or raises it.
- **Key** — the label itself. Matching is exact. A granted figure, a consumed
  figure, an observed overshoot, a display name, a message text or a rendering
  never participates in the key: a dimension granted 10 and a dimension granted
  1000 are one dimension, not two.
- **Key order** — lexicographic over the label, for collection comparison and
  duplicate detection only. It carries no causal meaning, expresses no
  precedence between dimensions, and is not the order in which dimensions are
  checked. No requirement authors an order over dimensions; each is enforced
  independently at its own charge point.
- **Causal order** — none between dimensions. Exhaustion on one dimension
  establishes no order relative to another, and the dimension reported by a
  resource-incomplete outcome is the one whose increment was refused, not the
  first in any ordering.
- **Absence** — a dimension with no caller-supplied limit has an absent budget.
  An absent budget compares only to absence. It is never defaulted to zero, to
  unbounded, to another dimension's grant, or to an ambient universal threshold.
  `NFR-002` bounds only "every *positive* caller-supplied" limit selected for the
  exact request, so an absent grant is not a grant of zero.
- **Digest domain** — not applicable; no component of this key is a digest. Where
  a budget figure is retained beside a digest in evidence, that digest's domain
  is selected under [FR-015](./FR-015-digest-domain-membership.md).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-016-AC-1 | Each member's value equals the exact label string in the table, and every dimension NFR-002 and FR-009 declare has exactly one member. | Inspection |
| FR-016-AC-2 | One unchanged input under a budget exactly sufficient on a dimension admits; the same input one unit short on that dimension returns that dimension's typed resource-incomplete outcome; and the same input under a fresh budget at the exact value admits again. | Test (TC-009, IT-001) |
| FR-016-AC-3 | A one-short outcome names exactly the dimension whose increment was refused and no other, and exhausting one dimension changes the granted or consumed figure of no other dimension. | Test (TC-009) |
| FR-016-AC-4 | No work is admitted after a dimension is exhausted, and no budget, partial consumption or accounted state carries from an exhausted attempt into the fresh-budget attempt. | Test (TC-009) |
| FR-016-AC-5 | A resource-incomplete outcome carries an identity distinct from every typed refusal identity, and is reported as neither a refusal of the input nor a pass. | Test (TC-009, TC-011) |
| FR-016-AC-6 | The granted and consumed figures on `elapsed-time` and `resolution-deadline` are recorded on every scenario that declares them, and neither consumed figure participates in the reproducibility comparison. | Test (TC-008, TC-009) |
| FR-016-AC-7 | A dimension with no caller-supplied limit compares only to absence, receives no default, and inherits no other dimension's grant. | Test (TC-009) |
| FR-016-AC-8 | Exceeding `resolution-entry-count`, `resolution-bytes-digested` or `resolution-deadline` returns that dimension's resource-incomplete outcome with no producer value read. | Test (TC-013) |

## Dependencies

- [NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md) enumerates the
  request dimensions, fixes inclusive maxima, the pre-work charge point, the
  monotonic deadline rule, and the typed resource-incomplete outcome.
- [FR-009](./FR-009-lock-external-producer-revisions.md) "Bounds" adds
  resolution's entry-count, bytes-digested and deadline dimensions.
- [NFR-005](../non-functional/NFR-005-retain-composed-acceptance-evidence.md)
  requires the granted and consumed figures to be retained per declared
  dimension and excludes elapsed time from the reproducibility comparison.
- [NFR-001](../non-functional/NFR-001-reproduce-semantic-results.md) requires the
  retained evidence to be reproducible, which is why elapsed time is excluded
  from the comparison.
- [FR-008](./FR-008-return-typed-refusals.md) owns the refusal envelope that a
  resource-incomplete outcome is held distinct from.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the kind and
  authority axes.

## Status

Draft. This artifact records existing meaning structurally and establishes no new
obligation. Where no requirement declares a dimension, none is invented here.

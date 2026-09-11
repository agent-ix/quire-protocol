---
id: SR-031
title: "Code review of the closed protocol identity newtype set"
type: SpecReview
analysis: code-review
scope: "src/ids.rs, tests/tc_002_identities.rs, spec/tests.md (TC-002 row)"
review_set: subset
evaluated_revision: "48db0398d60523f15eb97978d919673cf27cde40"
review_date: "2026-09-11"
relationships:
  - target: ix://agent-ix/quire-protocol/FR-002
    type: reviews
  - target: ix://agent-ix/quire-protocol/TC-002
    type: reviews
---
## Summary

Reviewed chunk C-07: the closed protocol identity set in `src/ids.rs` and its
TC-002 evidence in `tests/tc_002_identities.rs`. The change satisfies
FR-002-AC-1's identity-distinctness obligation — sixteen separate `struct`
types (FR-002-AC-1's thirteen plus FR-003-AC-3's registration, commit and
recovery identities), no type alias, no shared generic marker, equality and
hashing keyed on the producer's exact bytes, and no path from display text back
into an identity. Non-interchangeability is proved at compile time by nineteen
`compile_fail` documentation tests rather than asserted in prose. Four low
findings remain, all of them obligations on downstream chunks rather than
defects in the landed types.

## Verdict

**CONDITIONAL** — no high or medium findings; four low findings carry forward to
named downstream chunks.

## Assurance Context

- **Profile:** AP-001 Protocol implementation assurance profile
  (`spec/assurance/AP-001-protocol-implementation.md`, `profile_version: 0.2`,
  `status: proposed`). It applies: this change is part of the one exact
  `quire-protocol` Rust candidate the profile scopes, and `code-review` is a
  listed operation.
- **Applicable impact assessment:** `impact-false-conformance` — an invalid,
  open, incomplete, failed or exhausted assessment reported as complete
  conformance. Identity confusion is an upstream cause of exactly that: a
  conformance result attributed to the wrong send, delivery or effect is a
  false verdict about a real obligation. This chunk removes the compiler-level
  half of that risk.
- **Evaluated revision:** `48db0398d60523f15eb97978d919673cf27cde40` plus the
  C-07 working tree on branch `agent-b/c07-identity-newtypes`, based on
  `agent-b/c00-lib-facade`.
- **Context examined:** FR-002 (AC-1), FR-003 (AC-3, AC-4), TC-002, TM-001,
  A's `protocol_artifact::wire` record set at pinned rev
  `84975dd687d38d9b24eae84b7356e3f24cdc7b42`, and the AP-001 impact
  assessments.
- **Unavailable context:** no architecture selection record, no measurement
  evidence and no independence evidence exists for this repository yet. No
  structural-gap analysis is available for `src/ids.rs` because the modules that
  consume it (C-12, C-16, C-18, C-26) do not exist yet. Producer-reliance
  evidence for observation and temporal identities is unavailable because epic
  gate 4 is open.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Nothing in this module can force a consumer to read the producer's *identity* field rather than its display name: `SendId::new(name.as_bytes(), name)` type-checks. A's wire records expose both as plain `String`, so no type-level distinction exists to borrow, and the choice of source field necessarily belongs to the linker. C-12 must source identity bytes from the producer's identity field and must be tested for it; this module documents the obligation at the constructor and proves only that a rendered identity cannot be parsed back. | src/ids.rs:296 |
| FND-002 | low | `IdentityKind::ALL` and the type list in `every_type_id()` are two hand-written enumerations of one closed set, and stable Rust cannot compile-assert that either is complete. Mitigation in place: `tc_002_newtypes_are_not_interchangeable` fails if the two lists differ in length or if any kind or tag repeats, so a seventeenth identity added to one list and not the other goes red. Closing it properly needs a derive macro, which needs a `Cargo.toml` dependency, which COLLISION-7 serialises to one chunk per wave. | src/ids.rs:251 |
| FND-003 | low | The compile-fail evidence is a set of documentation tests, and a `compile_fail` doc test cannot carry an `ix_trace_rs::trace` attribute. The binding to TC-002/Task-002 is therefore indirect: `tc_002_compile_fail_proof_is_present` is trace-bound and asserts the nineteen cases and all sixteen expected-type positions are still present, so deleting or narrowing the proof fails a traced test. If B later adds trybuild under COLLISION-7's serialised dependency slot, these cases should move to a `tests/ui` lane whose harness test carries the binding directly. | tests/tc_002_identities.rs:186 |
| FND-004 | low | Identity bytes are retained as an unbounded `Vec<u8>` per identity, by design — FR-002-AC-1 requires exact retention and forbids truncation. There is consequently no per-identity length ceiling here; caller-controlled allocation at this site belongs to the NFR-002 resource inventory. C-30 must list `ids::*::new` as a retention site with its own boundary case. | src/ids.rs:296 |

## Checklist results

| Check | Result |
| --- | --- |
| Repo idioms | `#![forbid(unsafe_code)]` unchanged; `clippy::all = "deny"` honoured; `///` on every public item and a `//!` module header naming what the module owns and what it refuses to own. No new `#[allow]`, no new dependency. |
| Newtypes over primitives | Sixteen distinct `struct` types with private fields. No type alias and no `Id<Marker>` generic, either of which would make two identities unify. Proved by sixteen `compile_fail` pairs forming a closed cycle, so every kind appears once as the expected type and once as the wrong argument. |
| Positive control | One passing doc test calls every constructor with every parameter type, so each `compile_fail` case fails for the type mismatch and not for a typo in the harness — the standard failure mode of compile-fail tests. |
| Display text is not identity | `PartialEq`, `Eq`, `Ord` and `Hash` are hand-written over the identity bytes alone; deriving them would have made display text part of every map lookup. `tc_002_display_text_is_not_identity` asserts both directions and checks `HashMap` and `BTreeMap` keying, not just `==`. |
| No parse path | No `FromStr`, no `From<String>`, no `From<&str>` on any identity; three `compile_fail` cases prove it. A refusal message, a log line or a diagnostic therefore cannot be read back as an identity. |
| Exact retention | `new` copies bytes with no trim, case fold, Unicode normalisation or UTF-8 requirement. `tc_002_identity_bytes_are_retained_unchanged` covers embedded NUL, non-UTF-8 bytes, surrounding whitespace, case variants and NFC/NFD pairs, and asserts the near-miss forms stay distinct identities. |
| Wire round trip | `tc_002_wire_package_identities_round_trip_unchanged` builds a real `wire::Definition` and a real `wire::Requirement` and shows the retained bytes equal the producer's field byte for byte, and that two producer fields rendering under one display text remain two identities. |
| Public API boundary decision | Recorded explicitly in the module documentation and in the commit: `IdentityKind` is closed, not `#[non_exhaustive]`, because a `#[non_exhaustive]` enum forces downstream wildcard arms and a wildcard arm silently absorbs a later variant. The closed enum makes a seventeenth identity a compile error in every consumer — the intended cost of a specification amendment. |
| Exhaustiveness | The one `match` in the module (`IdentityKind::tag`) is exhaustive with no `_` arm. |
| Tautology check | No assertion restates a value the test set: the `TypeId` test fails if any two names resolve to one type, the display test fails if equality consults the rendering, and the retention test fails if any normalisation is introduced. |
| Panic surface | The library has no `unwrap`, `expect`, `panic!`, indexing, slicing, arithmetic, I/O or recursion. `unwrap` appears only in test code. |
| Boundary (G-BOUNDARY) | No grammar, parser, wire decoder, canonicalizer, runner, evidence store or producer semantics added. The module holds identity retention only, and offers no method that feeds identity bytes into a digest or canonical form (G-DIGEST). |
| Matrix honesty | Only the TC-002 row of TM-001 is touched, annotated to say the FR-002-AC-1 identity set is executing while AC-2..AC-4 are unwritten. The FR-002 row and the Test Execution Summary totals are left for the per-wave bookkeeping pass (COLLISION-4). |

## Validation and limits

Gates were run locally, serially, one Cargo job, warnings denied, each with
`CARGO_TARGET_DIR` unset so the shared target directory could not serve a stale
sibling artifact. Real counts are recorded in the pull request.

The `--features quire-extraction` lane named in G-GATES was not run: this
crate's `Cargo.toml` declares no `[features]` table and inventing one is out of
scope for this chunk. No hosted workflow was dispatched.

This review establishes the identity half of FR-002-AC-1 and the identity half
of FR-003-AC-3/AC-4 only. FR-002-AC-2, AC-3 and AC-4 need the linker (C-12,
C-13, C-14) and remain planned; the compensation algebra that consumes the
registration, commit and recovery identities is C-16 and remains planned.

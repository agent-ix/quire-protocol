---
id: NFR-006
title: "Fix the consumed dependency set and the accepted delivery surface"
type: NFR
quality_attribute: security
relationships:
  - target: ix://agent-ix/quire-protocol/FR-009
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-018
    type: constrains
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-004
    type: depends_on
---
# NFR-006: Fix the consumed dependency set and the accepted delivery surface

## Statement

The repository SHALL pin every package in its own build closure to an exact immutable identifier recorded in a committed lockfile.
The repository SHALL enumerate every closure package that executes code at build time and reconcile that enumeration against the lockfile in both directions.
The library SHALL accept from a producer delivery only the exact byte sequences a `pinned` version-lock entry's recorded digest covers.
The repository SHALL record, for every git-sourced dependency, the same revision its own build manifest pins.

## Scope

Applies to two integrity surfaces this repository actually has, and to nothing
else:

1. **Its own build closure.** Every entry of `[dependencies]` and
   `[dev-dependencies]` in `Cargo.toml`, every package in `Cargo.lock`, and the
   rights record in `DEPENDENCIES.md` that names the same revisions.
2. **What it accepts from a producer delivery.** The bytes reached through a
   `pinned` [FR-009](../functional/FR-009-lock-external-producer-revisions.md)
   entry, and the delivery root those bytes sit in.

Does not apply to:

- The version-lock entry's own shape, key, status set or independence rules.
  [FR-012](../functional/FR-012-version-lock-entry.md) owns the entry and
  [FR-009](../functional/FR-009-lock-external-producer-revisions.md) owns
  resolution, digest recomputation, the two-status model, the owner-supplied
  immutability assertion, the self-authorization prohibition and the five-stage
  gate order. This requirement adds no status, no key component and no gate
  stage, and states no rule about how a digest is computed.
- Whether a producer's revision is genuinely immutable or genuinely published.
  FR-009 makes that an assertion the owner records, not a property this library
  checks, and this requirement does not convert it into one.
- Reading a version-control history, spawning a process, or accessing a network
  in order to establish any of the above. `AD-001` Views and
  [NFR-003](./NFR-003-preserve-rust-assurance-boundary.md) place those outside
  the boundary, and FR-009-AC-7 already verifies their absence.

### Boundary against the neighbouring non-functional requirements

| Neighbour | What it owns | What this requirement owns instead |
| --- | --- | --- |
| [NFR-002](./NFR-002-enforce-resource-bounds.md) | The budget dimensions resolution consumes, and the typed resource-incomplete outcome when one is exhausted. | Which bytes and which packages are admissible at all. A closure that is correctly bounded can still be unpinned. |
| [NFR-003](./NFR-003-preserve-rust-assurance-boundary.md) | That first-party executable paths are Rust, that no local producer runner or evidence store exists, and that the closure contains no such implementation. | That the closure, whatever it contains, is fixed to exact identifiers and that its build-time execution surface is enumerated. NFR-003 asks *what kind of thing* a dependency is; this asks *whether it can change under us*. |
| [NFR-004](./NFR-004-define-local-verification-gate.md) | Which commands the local gate is, in which order, in how many configuration lanes. | That a gate run's evidence is invalid if the closure moved during it. This requirement adds no gate command and changes no lane count. |
| [NFR-005](./NFR-005-retain-composed-acceptance-evidence.md) | What a scenario must carry to its handoff, including the resolution state it ran against. | That the resolution state was resolvable against a fixed closure in the first place. NFR-005 retains a state; this constrains what may enter it. |

## Rationale

FR-009 pins the *external producers* B consumes. Nothing pins B. This
repository's own build closure is 196 lockfile packages, seven of them git
sources carrying a revision and no content checksum. The rights record in
`DEPENDENCIES.md` has drifted from the manifest twice on the same edge: first
naming `84975dd687d38d9b24eae84b7356e3f24cdc7b42` against a manifest pinning
`5690c2b29c155ec72e21f5a7bb00fe1146b6f71a`, then trailing at `5690c2b` after
the manifest moved to `04254a31071cc2d4c97e5c0707e1414141e55d89`, while
`agent-ix-baseline-producer` — a fourth git source the newer revision drags in
transitively — went unrecorded entirely. Both are corrected in the record, and
the recurrence is the argument for this requirement: the drift is structural,
not a one-off transcription slip. A
consumer that verifies a producer's bytes to the byte while resolving its own
compiler input from a moving reference has verified the wrong half of the
supply chain.

The second half is the delivery surface. FR-009 says what a version-lock entry
covers; FR-018 says what a delivery index records; neither says what happens to
a file that is *in the delivery root and covered by neither*. FR-018-AC-1
establishes that the index cannot widen acceptance, which leaves the question
unanswered rather than answered. An unenumerated file in a pinned delivery root
is the shape of an offer that arrives without being accepted.

Both halves are B-ownable today and need no producer.

## Measurement and Evaluation

Every metric is denominated on the closure and delivery the repository actually
declares, so an unpinned entry lowers the score rather than being omitted from
it.

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Declared manifest dependencies pinned to an exact `=` version or an exact 40-hex git `rev` | Every declared dependency | 100%, 0 range, branch, tag or path requirements | Inspection |
| Lockfile packages carrying either a `checksum` or a 40-hex git `rev` equal to the manifest's | Every package but this crate | 195 of 195 today | Inspection |
| Closure packages executing code at build time that are absent from the build-execution enumeration, or enumerated but absent from the closure | None | 0 unmatched in either direction | Inspection |
| Git-sourced dependency revisions where the rights record and the build manifest disagree | None | 0 disagreements today | Inspection |
| Gate runs whose lockfile differs before and after the run | None | 0 accepted as evidence | Inspection |
| Delivery-root bytes reaching a consumer path without a `pinned` entry's digest covering them | None | 0 bytes | Test (TC-013) |
| Version-lock entries a delivery index supplied, selected or repaired | None | 0 entries | Test (TC-013) |

## Verification

The closure metrics are read from the repository's own Cargo manifest and
lockfile and from `DEPENDENCIES.md`. No metric requires a network, a registry
query or a version-control history: a pin is compared against a recorded
identifier, never resolved.

Retain a build-execution enumeration naming every closure package that runs code
during a build — a package shipping a build script, and a package declaring
itself a procedural macro — together with the declared dependency that pulls it
in. Reconcile the enumeration against the lockfile in both directions, so a
package present in one and absent from the other is a finding either way.

Each criterion below carries its own negative control, stated as the exact
mutation the check must refuse. A criterion whose mutation the check accepts is
failed, and a criterion with no runnable mutation is recorded as unexecutable
rather than as passing. The delivery-surface controls run on the B-owned
fixtures TC-013 already uses; they require no producer delivery, because a
B-authored fixture root can carry an unenumerated file.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-006-AC-1 | Every declared dependency and dev-dependency resolves through an exact `=` version requirement or an exact 40-hex git `rev`; introducing a caret or range requirement, a `branch`, a `tag`, or a `path` source for any one of them is refused, and the refusal names the offending dependency. | Inspection |
| NFR-006-AC-2 | Every lockfile package other than this crate carries either a `checksum` or a git `rev` byte-equal to the revision its manifest entry pins; deleting one `checksum` line, or changing one lockfile `rev` so it differs from the manifest's, is refused and names that package. | Inspection |
| NFR-006-AC-3 | The build-execution enumeration and the lockfile agree in both directions; adding a closure package that runs a build script or is a procedural macro without enumerating it, and leaving an enumerated entry whose package no longer appears in the closure, are each refused and each names the unmatched side. | Inspection |
| NFR-006-AC-4 | Every git-sourced dependency's revision in the rights record equals the revision the resolved lockfile carries, and no git source in the lockfile is absent from the record; a divergence or an omission is refused and names the package and both revisions. The criterion covers transitive git sources, not only the manifest's direct pins. | Inspection |
| NFR-006-AC-5 | A verification run whose lockfile is not byte-identical before and after it is not accepted as evidence of the closure; introducing a dependency change that the run would resolve rather than refuse is detected by that comparison alone, independently of which commands [NFR-004](./NFR-004-define-local-verification-gate.md) enumerates. | Inspection |
| NFR-006-AC-6 | A byte sequence in a pinned delivery root that no `pinned` entry's recorded digest covers does not reach a consumer path, and adding one unenumerated file to an otherwise complete fixture root returns a typed refusal naming that file rather than admitting the delivery. | Test (TC-013) |
| NFR-006-AC-7 | Swapping one delivery member's bytes for different bytes at the same path returns a typed refusal, and the substituted bytes reach no consumer path; the refusal is distinct from the refusal an absent member produces. | Test (TC-013) |
| NFR-006-AC-8 | A delivery index supplies, selects or repairs no version-lock entry and widens no accepted byte set; editing an index to name a file the manifest does not cover changes no acceptance outcome, and deleting the index changes no acceptance outcome either. | Test (TC-017) |

## Dependencies

- **Upstream**: [FR-009](../functional/FR-009-lock-external-producer-revisions.md)
  supplies the version-lock manifest, the `pinned`/`unresolved` status model, and
  the digest recomputed over the exact delivered bytes that AC-6 and AC-7 compare
  against. [FR-012](../functional/FR-012-version-lock-entry.md) owns the entry's
  `(owner, contract version)` key and its `content_digest` component.
- **Constrains**: [FR-018](../functional/FR-018-delivery-index.md), whose index
  AC-8 holds to the non-authority FR-018-AC-1 already states, and which this
  requirement forbids from widening the accepted byte set.
- **Boundary**: [NFR-003](./NFR-003-preserve-rust-assurance-boundary.md) owns the
  containment that keeps every metric here offline;
  [NFR-004](./NFR-004-define-local-verification-gate.md) owns the gate commands
  AC-5 declines to duplicate.
- **Downstream**: [TM-001](../tests.md) carries this requirement and TC-013's
  Traceability names its delivery-surface criteria. Task-008 performs the final
  bidirectional source/rights reconciliation after every implementation pin.

## Status

In progress. The current lock has 195 non-root packages and seven git sources;
Cargo/rights records cover the current exact pins, including QSL `2736b62`,
QObs `9240063`, qverify `293e342` and its transitive Engineering Assurance
`9237ce5`.
Task-013 implements TC-013's exact-member controls for AC-6 and AC-7;
Task-019/#66 owns the missing TC-017 consumer-index control for AC-8. Every
later pin must update Cargo.lock and DEPENDENCIES.md in the same ticket before
this requirement can close.
